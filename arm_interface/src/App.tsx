import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";
import { useArm } from "./providers/ArmProvider";
import { Box, Center, CircularProgress, Stack, Text } from "@chakra-ui/react";
import { VisualizationComponent } from "./components/Visualization.component";
import { ManualControlComponent } from "./components/ManualControl.component";
import { Vector3 } from "three";

export enum EControlMode {
  TranslateEndEffector = "Translate end-effector",
  RotateEndEffector = "Rotate end-effector",
}

export interface ITranslateEndEffectorControlMode {
  mode: EControlMode.TranslateEndEffector;
  target_position: Vector3;
}

export interface IRotateEndEffectorControlMode {
  mode: EControlMode.RotateEndEffector;
  euler_angles: Vector3;
}

export type TControlMode =
  | ITranslateEndEffectorControlMode
  | IRotateEndEffectorControlMode;

function App() {
  const { isLoading: isArmLoading } = useArm();
  const [controlMode, setControlMode] = useState<TControlMode>({
    mode: EControlMode.TranslateEndEffector,
    target_position: new Vector3(0, 0, 0),
  });

  if (isArmLoading) {
    return (
      <Center>
        <Stack>
          <Text>Loading</Text>
          <CircularProgress isIndeterminate={true} />
        </Stack>
      </Center>
    );
  }

  return (
    <Box height={"100vh"}>
      <VisualizationComponent controlMode={controlMode} setControlMode={setControlMode} />
      <Box position={"absolute"} top={8} right={8}>
        <ManualControlComponent controlMode={controlMode} setControlMode={setControlMode} />
      </Box>
    </Box>
  );
}

export default App;
