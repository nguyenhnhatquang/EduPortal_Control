import type { DeployProgressEvent, Settings } from "../../types";
import { fallbackSettings } from "../settings/defaults";
import type { DeployStepState, DeployStepView } from "./types";

const deployStepOrder = ["download", "validate", "extract", "npm", "asset", "config", "pm2", "history", "cleanup"];

export function createDeploySteps(settings: Settings = fallbackSettings): DeployStepView[] {
  return [
    {
      id: "validate",
      label: "Validate package",
      detail: "Check Portal/build/index.js and WebApi/WebApi.exe.",
      state: "pending",
    },
    {
      id: "extract",
      label: "Extract release",
      detail: `Create a new timestamped folder under ${settings.deployRoot}.`,
      state: "pending",
    },
    {
      id: "npm",
      label: "Install Portal dependencies",
      detail: settings.portalInstallDependencies ? "Run npm install --omit=dev in Portal." : "Disabled in settings.",
      state: settings.portalInstallDependencies ? "pending" : "skipped",
    },
    {
      id: "asset",
      label: "Install Portal assets",
      detail: settings.portalAssetCopy.enabled
        ? `Copy or extract ${settings.portalAssetCopy.source} to ${settings.portalAssetCopy.destination}.`
        : "Disabled in settings.",
      state: settings.portalAssetCopy.enabled ? "pending" : "skipped",
    },
    {
      id: "config",
      label: "Generate PM2 config",
      detail: "Write config.json with Portal/WebApi scripts, logs, and env.",
      state: "pending",
    },
    {
      id: "pm2",
      label: "Reload PM2",
      detail: "Run pm2 startOrReload config.json --update-env.",
      state: "pending",
    },
    {
      id: "history",
      label: "Save deployment history",
      detail: "Mark active deployment and apply retention.",
      state: "pending",
    },
  ];
}

export function createPortalReleaseDeploySteps(settings: Settings = fallbackSettings): DeployStepView[] {
  return [
    {
      id: "download",
      label: "Download Portal release",
      detail: "Download the selected GitHub release asset.",
      state: "pending",
    },
    ...createDeploySteps(settings),
    {
      id: "cleanup",
      label: "Clean release zip",
      detail: "Delete the downloaded GitHub release zip from local cache.",
      state: "pending",
    },
  ];
}

export function markDeployStep(steps: DeployStepView[], id: string, state: DeployStepState): DeployStepView[] {
  let seenTarget = false;
  return steps.map((step) => {
    if (step.id === id) {
      seenTarget = true;
      return { ...step, state };
    }
    if (!seenTarget && state === "running" && step.state === "pending") {
      return { ...step, state: "done" as DeployStepState };
    }
    return step;
  });
}

export function failActiveDeployStep(steps: DeployStepView[]): DeployStepView[] {
  if (steps.some((step) => step.state === "failed")) {
    return steps;
  }

  const runningIndex = steps.findIndex((step) => step.state === "running");
  if (runningIndex >= 0) {
    return steps.map((step, index) => (index === runningIndex ? { ...step, state: "failed" } : step));
  }

  const firstPendingIndex = steps.findIndex((step) => step.state === "pending");
  if (firstPendingIndex >= 0) {
    return steps.map((step, index) => (index === firstPendingIndex ? { ...step, state: "failed" } : step));
  }

  return steps;
}

export function applyDeployProgressEvent(steps: DeployStepView[], event: DeployProgressEvent): DeployStepView[] {
  const baseSteps = shouldResetForNewRun(steps, event) ? resetDeployStepStates(steps) : steps;
  let seenTarget = false;
  const nextSteps = baseSteps.map((step) => {
    if (step.id === event.stepId) {
      seenTarget = true;
      return {
        ...step,
        label: event.label,
        detail: event.detail,
        state: event.state,
      };
    }

    if (!seenTarget && event.state === "running" && step.state === "pending") {
      return { ...step, state: "done" as DeployStepState };
    }

    return step;
  });

  if (nextSteps.some((step) => step.id === event.stepId)) {
    return nextSteps;
  }

  return insertDeployStep(nextSteps, {
    id: event.stepId,
    label: event.label,
    detail: event.detail,
    state: event.state,
  });
}

function shouldResetForNewRun(steps: DeployStepView[], event: DeployProgressEvent) {
  if (event.state !== "running") return false;
  if (!steps.some((step) => step.state === "done" || step.state === "failed")) return false;
  if (event.stepId === "download") return true;
  return event.stepId === "validate" && !steps.some((step) => step.id === "download");
}

function resetDeployStepStates(steps: DeployStepView[]): DeployStepView[] {
  return steps.map((step) => ({
    ...step,
    state: step.state === "skipped" ? step.state : ("pending" as DeployStepState),
  }));
}

function insertDeployStep(steps: DeployStepView[], step: DeployStepView): DeployStepView[] {
  const targetOrder = deployStepOrder.indexOf(step.id);
  if (targetOrder < 0) {
    return [...steps, step];
  }

  const insertAt = steps.findIndex((existing) => {
    const existingOrder = deployStepOrder.indexOf(existing.id);
    return existingOrder >= 0 && existingOrder > targetOrder;
  });

  if (insertAt < 0) {
    return [...steps, step];
  }

  return [...steps.slice(0, insertAt), step, ...steps.slice(insertAt)];
}

export function buildDeployStepsFromResult(
  settings: Settings,
  result: {
    postDeploy: Array<{ name: string; skipped: boolean; success: boolean; message: string }>;
    pm2: { success: boolean; skipped: boolean; message: string };
  },
  options: { portalRelease?: boolean } = {},
): DeployStepView[] {
  const steps = (options.portalRelease ? createPortalReleaseDeploySteps(settings) : createDeploySteps(settings)).map((step) => ({
    ...step,
    state: step.state === "skipped" ? ("skipped" as DeployStepState) : ("done" as DeployStepState),
  }));
  const npmStep = result.postDeploy.find((step) => step.name === "Portal npm install");
  const assetStep = result.postDeploy.find((step) => step.name === "Portal asset copy");

  return steps.map((step) => {
    if (step.id === "npm" && npmStep) {
      return {
        ...step,
        detail: npmStep.message,
        state: npmStep.skipped ? ("skipped" as DeployStepState) : npmStep.success ? "done" : "failed",
      };
    }
    if (step.id === "asset" && assetStep) {
      return {
        ...step,
        detail: assetStep.message,
        state: assetStep.skipped ? ("skipped" as DeployStepState) : assetStep.success ? "done" : "failed",
      };
    }
    if (step.id === "pm2") {
      return {
        ...step,
        detail: result.pm2.message,
        state: result.pm2.skipped ? ("skipped" as DeployStepState) : result.pm2.success ? "done" : "failed",
      };
    }
    return step;
  });
}

export function currentDeployStepLabel(steps: DeployStepView[]) {
  const failed = steps.find((step) => step.state === "failed");
  if (failed) return `${failed.label} failed`;
  const running = steps.find((step) => step.state === "running");
  if (running) return `${running.label} is running`;
  const completed = steps.filter((step) => step.state === "done" || step.state === "skipped").length;
  return `${completed}/${steps.length} step(s) completed`;
}
