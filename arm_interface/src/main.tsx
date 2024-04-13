import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles.css";
import { ArmProvider } from "./providers/ArmProvider";
import { ChakraProvider } from "@chakra-ui/react";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ChakraProvider>
      <ArmProvider>
        <App />
      </ArmProvider>
    </ChakraProvider>
  </React.StrictMode>,
);
