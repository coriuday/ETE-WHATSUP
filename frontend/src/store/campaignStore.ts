import { create } from "zustand";
import { Campaign } from "@/types";

interface CampaignState {
  currentStep: number;
  draftCampaign: Partial<Campaign> & {
    selectedContactIds: string[];
    selectedSegmentId?: string;
    variables: Record<string, string>;
  };
  setStep: (step: number) => void;
  nextStep: () => void;
  prevStep: () => void;
  updateDraft: (data: Partial<CampaignState["draftCampaign"]>) => void;
  resetDraft: () => void;
}

const initialDraft = {
  name: "",
  type: "promotional" as const,
  selectedContactIds: [],
  variables: {},
};

export const useCampaignStore = create<CampaignState>((set) => ({
  currentStep: 0,
  draftCampaign: initialDraft,
  
  setStep: (step) => set({ currentStep: step }),
  nextStep: () => set((state) => ({ currentStep: state.currentStep + 1 })),
  prevStep: () => set((state) => ({ currentStep: Math.max(0, state.currentStep - 1) })),
  
  updateDraft: (data) =>
    set((state) => ({
      draftCampaign: { ...state.draftCampaign, ...data },
    })),
    
  resetDraft: () => set({ currentStep: 0, draftCampaign: initialDraft }),
}));
