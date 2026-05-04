export type DeployStepState = "pending" | "running" | "done" | "failed" | "skipped";

export interface DeployStepView {
  id: string;
  label: string;
  detail: string;
  state: DeployStepState;
}
