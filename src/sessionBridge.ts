/**
 * MCP session bridge publisher + live writeback poll.
 *
 * Desktop (Tauri): invokes native `mcp_session_bridge_*` commands.
 * Browser/WASM: uses Vite `/__nbcad_session/*` middleware against NBCAD_SESSION_DIR.
 *
 * Live mode: UI publishes model/focus/heartbeat; MCP may attach with writeback and
 * bump generation. The UI polls and applies external revisions via
 * `applyExternalProjectModel`.
 */
import { invoke, isTauri } from '@tauri-apps/api/core';
import { getEngine } from './engine';
import { applyExternalProjectModel } from './files/projectTabs';
import {
  exportProjectModelWithVisibility,
  useAppStore,
  type AppMode,
  type SketchTool,
} from './store/appStore';

export type McpFocusPack =
  | 'document'
  | 'sketch'
  | 'solid'
  | 'modify'
  | 'body_ops'
  | 'datums'
  | 'history'
  | 'inspect'
  | 'print'
  | 'drawing';

/** Keep in sync with mcp-server/src/disclosure.rs focus packs. */
export function focusFromUi(
  mode: AppMode,
  activeTool: SketchTool,
  solidDialog: string | null,
  activeTab?: string,
): McpFocusPack {
  if (activeTab === 'drawing') return 'drawing';
  if (solidDialog) {
    switch (solidDialog) {
      case 'fillet':
      case 'chamfer':
      case 'hole':
        return 'modify';
      case 'shell':
      case 'mirror':
      case 'rectangular_pattern':
      case 'circular_pattern':
      case 'combine':
      case 'split_body':
        return 'body_ops';
      case 'construction_plane':
        return 'datums';
      case 'extrude':
      case 'revolve':
      case 'sweep':
      case 'loft':
      case 'rib':
        return 'solid';
      default:
        return 'solid';
    }
  }
  if (mode === 'sketch') return 'sketch';
  if (mode === 'pickPlane') return 'datums';
  if (mode === 'solid') return 'solid';
  if (activeTool) return 'sketch';
  return 'document';
}

function activeSolidDialog(state: ReturnType<typeof useAppStore.getState>): string | null {
  if (state.filletDialogFeature !== null) return 'fillet';
  if (state.chamferDialogFeature !== null) return 'chamfer';
  if (state.holeDialogFeature !== null) return 'hole';
  if (state.extrudeDialogFeature !== null) return 'extrude';
  if (state.revolveDialogFeature !== null) return 'revolve';
  if (state.sweepDialogFeature !== null) return 'sweep';
  if (state.loftDialogFeature !== null) return 'loft';
  if (state.ribDialogFeature !== null) return 'rib';
  if (state.constructionPlaneDialog) return 'construction_plane';
  if (state.bodyFeatureDialog) return state.bodyFeatureDialog.kind;
  return null;
}

let publishTimer: ReturnType<typeof setTimeout> | null = null;
let heartbeatTimer: ReturnType<typeof setInterval> | null = null;
let pollTimer: ReturnType<typeof setInterval> | null = null;
let started = false;
let applyingExternal = false;
let lastSeenGeneration = 0;
let knownSessionId: string | null = null;
let skipPublishUntil = 0;

interface PublishReservation {
  session_id: string;
  generation: number;
}

function useHttpBridge(): boolean {
  return !isTauri();
}

async function httpJson(url: string, init?: RequestInit): Promise<unknown> {
  const response = await fetch(url, {
    ...init,
    headers: {
      'Content-Type': 'application/json',
      ...(init?.headers ?? {}),
    },
  });
  const body = await response.json().catch(() => ({}));
  if (!response.ok) {
    throw new Error(
      typeof (body as { error?: string }).error === 'string'
        ? (body as { error: string }).error
        : `session bridge HTTP ${response.status}`,
    );
  }
  return body;
}

async function reserveGeneration(): Promise<PublishReservation> {
  if (useHttpBridge()) {
    const result = (await httpJson('/__nbcad_session/reserve', {
      method: 'POST',
      body: JSON.stringify({ window: 'browser' }),
    })) as PublishReservation;
    return result;
  }
  return invoke<PublishReservation>('mcp_session_bridge_reserve');
}

async function writeSnapshot(payload: {
  focus: string;
  model_json: string | null;
  active_sketch_json?: string | null;
  generation: number;
}): Promise<void> {
  if (useHttpBridge()) {
    await httpJson('/__nbcad_session/write', {
      method: 'POST',
      body: JSON.stringify({ window: 'browser', ...payload }),
    });
    return;
  }
  await invoke('mcp_session_bridge_write', {
    payload: JSON.stringify(payload),
  });
}

async function writeHeartbeat(): Promise<void> {
  if (useHttpBridge()) {
    await httpJson('/__nbcad_session/heartbeat', {
      method: 'POST',
      body: JSON.stringify({ window: 'browser' }),
    });
    return;
  }
  await invoke('mcp_session_bridge_heartbeat');
}

async function writerIsMcp(sessionId: string | null): Promise<boolean> {
  if (!sessionId) return false;
  try {
    if (useHttpBridge()) {
      const writer = (await httpJson(`/__nbcad_session/${sessionId}/writer.json`)) as {
        writer?: string;
      };
      return writer.writer === 'mcp';
    }
    const snapshot = await invoke<{ writer?: { writer?: string } }>('mcp_session_bridge_poll');
    return snapshot.writer?.writer === 'mcp';
  } catch {
    return false;
  }
}

async function publishNow(): Promise<void> {
  if (applyingExternal) return;
  if (Date.now() < skipPublishUntil) return;
  const state = useAppStore.getState();
  // Browser always publishes when the HTTP bridge is available; Tauri when native.
  if (!useHttpBridge() && state.engineKind !== 'tauri') return;
  if (state.mode === 'sketch' || state.activeTool) return;
  // Respect live writer lock: do not overwrite while MCP holds the session.
  if (await writerIsMcp(knownSessionId)) return;
  const focus = focusFromUi(
    state.mode,
    state.activeTool,
    activeSolidDialog(state),
    state.activeTab,
  );
  try {
    const engine = await getEngine();
    const activeSketch = await engine.activeSketch();
    let modelJson: string | null = null;
    try {
      const model = await exportProjectModelWithVisibility(engine);
      modelJson = typeof model === 'string' ? model : JSON.stringify(model);
    } catch (error) {
      // A half-finished sketch must not enter the persisted project format,
      // but diagnostics still need the live entity/constraint snapshot. The
      // native bridge keeps its previous completed model.json beside it.
      if (activeSketch === null) throw error;
    }
    const reservation = await reserveGeneration();
    await writeSnapshot({
      focus,
      model_json: modelJson,
      active_sketch_json: activeSketch === null ? null : JSON.stringify(activeSketch),
      generation: reservation.generation,
    });
    knownSessionId = reservation.session_id;
    lastSeenGeneration = Math.max(lastSeenGeneration, reservation.generation);
  } catch (error) {
    console.debug('[sessionBridge] publish failed', error);
  }
}

async function heartbeatNow(): Promise<void> {
  const state = useAppStore.getState();
  if (!useHttpBridge() && state.engineKind !== 'tauri') return;
  try {
    await writeHeartbeat();
  } catch (error) {
    console.debug('[sessionBridge] heartbeat failed', error);
  }
}

async function pollWriteback(): Promise<void> {
  if (!knownSessionId || applyingExternal) return;
  const state = useAppStore.getState();
  if (state.mode === 'sketch' || state.activeTool || state.solidBusy || state.projectBusy) {
    return;
  }
  try {
    let generation = 0;
    let source: string | undefined;
    let modelJson: string | null = null;
    let workspaceHint: string | undefined;
    if (useHttpBridge()) {
      const heartbeat = (await httpJson(
        `/__nbcad_session/${knownSessionId}/heartbeat.json`,
      )) as { generation?: number; source?: string };
      generation = Number(heartbeat.generation ?? 0);
      source = heartbeat.source;
      if (generation <= lastSeenGeneration) return;
      if (source !== 'mcp') {
        lastSeenGeneration = generation;
        return;
      }
      const model = await httpJson(`/__nbcad_session/${knownSessionId}/model.json`);
      modelJson = typeof model === 'string' ? model : JSON.stringify(model);
      try {
        const focus = (await httpJson(
          `/__nbcad_session/${knownSessionId}/focus.json`,
        )) as { workspace?: string; focus?: string };
        workspaceHint =
          focus.workspace === 'drawing' || focus.focus === 'drawing' ? 'drawing' : 'solid';
      } catch {
        workspaceHint = 'solid';
      }
    } else {
      const snapshot = await invoke<{
        generation?: number;
        source?: string;
        model_json?: string;
        focus?: { workspace?: string; focus?: string };
      }>('mcp_session_bridge_poll');
      generation = Number(snapshot.generation ?? 0);
      source = snapshot.source;
      if (generation <= lastSeenGeneration) return;
      if (source !== 'mcp') {
        lastSeenGeneration = generation;
        return;
      }
      modelJson = snapshot.model_json ?? null;
      workspaceHint =
        snapshot.focus?.workspace === 'drawing' || snapshot.focus?.focus === 'drawing'
          ? 'drawing'
          : 'solid';
    }
    if (!modelJson) return;
    applyingExternal = true;
    try {
      await applyExternalProjectModel(modelJson, workspaceHint);
      lastSeenGeneration = generation;
      skipPublishUntil = Date.now() + 1_000;
      if (typeof window !== 'undefined') {
        (window as Window & { __nbcadLastMcpGeneration?: number }).__nbcadLastMcpGeneration =
          generation;
      }
    } finally {
      applyingExternal = false;
    }
  } catch (error) {
    console.debug('[sessionBridge] poll failed', error);
  }
}

export function scheduleSessionBridgePublish(): void {
  if (publishTimer) clearTimeout(publishTimer);
  publishTimer = setTimeout(() => {
    publishTimer = null;
    void publishNow();
  }, 300);
}

export function startSessionBridge(): void {
  if (started) return;
  started = true;
  useAppStore.subscribe((state, prev) => {
    if (
      state.document !== prev.document ||
      state.solidScene !== prev.solidScene ||
      state.drawingDocument !== prev.drawingDocument ||
      state.activeTab !== prev.activeTab ||
      state.activeSketch !== prev.activeSketch ||
      state.mode !== prev.mode ||
      state.activeTool !== prev.activeTool ||
      activeSolidDialog(state) !== activeSolidDialog(prev)
    ) {
      scheduleSessionBridgePublish();
    }
  });
  scheduleSessionBridgePublish();
  if (heartbeatTimer) clearInterval(heartbeatTimer);
  heartbeatTimer = setInterval(() => {
    void heartbeatNow();
  }, 10_000);
  if (pollTimer) clearInterval(pollTimer);
  pollTimer = setInterval(() => {
    void pollWriteback();
  }, 500);
}

/** Test/helper: force an immediate publish and return the session id. */
export async function publishSessionBridgeNow(): Promise<string | null> {
  await publishNow();
  return knownSessionId;
}

/** Test/helper: watch an existing session id for MCP writeback (skip OCCT publish). */
export function watchSessionBridge(sessionId: string, generation = 0): void {
  knownSessionId = sessionId;
  lastSeenGeneration = generation;
}
