import type { MaintenanceGuide, MaintenanceGuideStep } from "./types";

export const SAFETY_STEP_ID = "proexel-safety-lockout";
export const BEFORE_PHOTO_STEP_ID = "proexel-visual-before";
export const AFTER_PHOTO_STEP_ID = "proexel-work-after";

const RESERVED_STEP_IDS = new Set([SAFETY_STEP_ID, BEFORE_PHOTO_STEP_ID, AFTER_PHOTO_STEP_ID]);

export function operationalGuide(source: MaintenanceGuide): MaintenanceGuide {
  const steps: MaintenanceGuideStep[] = [
    standardStep(
      SAFETY_STEP_ID,
      "Secure the machine",
      "Apply lockout and tagout, isolate every energy source and confirm zero energy before intervention.",
      "confirmation",
      "Do not proceed until the machine is safely isolated.",
    ),
    standardStep(
      BEFORE_PHOTO_STEP_ID,
      "Visual inspection and initial photo",
      "Inspect the component and take a clear photo showing its condition before maintenance.",
      "photo",
    ),
    ...source.steps.filter((step) => !RESERVED_STEP_IDS.has(step.id)).map((step) => structuredClone(step)),
    standardStep(
      AFTER_PHOTO_STEP_ID,
      "Photo of completed work",
      "Take a clear photo of the completed maintenance before concluding this component.",
      "photo",
    ),
  ];
  return { version: source.version, steps: steps.map((step, order) => ({ ...step, order })) };
}

function standardStep(
  id: string,
  title: string,
  instructions: string,
  stepType: MaintenanceGuideStep["step_type"],
  safetyWarning: string | null = null,
): MaintenanceGuideStep {
  return {
    id,
    title,
    description: null,
    instructions,
    step_type: stepType,
    required: true,
    reference_photo_ids: [],
    safety_warning: safetyWarning,
    expected_value: null,
    options: [],
    order: 0,
  };
}
