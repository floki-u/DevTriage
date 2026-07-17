import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { Detail } from "./Detail";
import "./styles.css";

const isDetailView = new URLSearchParams(window.location.search).get("view") === "detail";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    {isDetailView ? <Detail /> : <App />}
  </StrictMode>
);
