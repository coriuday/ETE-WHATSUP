"use client";

import type { ReactNode } from "react";
import { ThemeProvider, createTheme } from "@mui/material/styles";
import { LocalizationProvider } from "@mui/x-date-pickers";
import { AdapterDateFns } from "@mui/x-date-pickers/AdapterDateFns";

const theme = createTheme({
  palette: {
    primary: { main: "#14B8A6" },
    text: { primary: "#0F3D3A", secondary: "#4A6B68" },
    divider: "#D5E4E1",
    background: { default: "#F6FAF9", paper: "#FFFFFF" },
  },
  shape: { borderRadius: 14 },
  typography: {
    fontFamily: "var(--font-sans), Inter, system-ui, sans-serif",
    fontSize: 13,
  },
  components: {
    MuiPaper: {
      styleOverrides: {
        root: { boxShadow: "none", border: "1px solid #D5E4E1" },
      },
    },
  },
});

export function MuiProvider({ children }: { children: ReactNode }) {
  return (
    <ThemeProvider theme={theme}>
      <LocalizationProvider dateAdapter={AdapterDateFns}>{children}</LocalizationProvider>
    </ThemeProvider>
  );
}
