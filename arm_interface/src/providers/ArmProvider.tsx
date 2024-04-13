import React, { useContext } from "react";
import { useListen } from "../hooks/useListen";
import { invoke } from "@tauri-apps/api";
import { timeout } from "../helpers/timeout";
import { Vector3 } from "three";

export interface IKinematicParameters {
  l_0: number;
  l_1: number;
  l_2: number;
  l_3: number;
  l_4: number;
}

export interface IKinematicState {
  theta_0: number;
  theta_1: number;
  theta_2: number;
  theta_3: number;
  theta_4: number;
}

export type TArmVertices = number[][];

export interface IArmStateChangedEvent {
  kinematicState: IKinematicState;
  vertices: TArmVertices;
}

export interface IGetVerticesResponse {
  vertices: TArmVertices;
}

export interface IGetKinematicStateResponse {
  kinematicState: IKinematicState;
}

export interface IArmContext {
  vertices: Vector3[];
  kinematicState: IKinematicState;
  kinematicParameters: IKinematicParameters;
  isLoading: boolean;
}

export interface IVariableArmState {
  vertices: Vector3[];
  kinematicState: IKinematicState;
}

export const ArmContext = React.createContext<IArmContext>({} as IArmContext);

export interface IArmProviderProps {
  children: any;
}

export const useArm = (): IArmContext => {
  return useContext<IArmContext>(ArmContext);
};

export const ArmProvider = ({
  children,
}: IArmProviderProps): React.ReactElement => {
  // Keep track of the kinematic parameters.
  const [kinematicParameters, setKinematicParameters] =
    React.useState<IKinematicParameters>({
      l_0: 0.0,
      l_1: 0.0,
      l_2: 0.0,
      l_3: 0.0,
      l_4: 0.0,
    });

  // Keeps track of the vertices and kinematic state (these change).
  const [{ kinematicState, vertices }, setVariableArmState] =
    React.useState<IVariableArmState>({
      kinematicState: {
        theta_0: 0.0,
        theta_1: 0.0,
        theta_2: 0.0,
        theta_3: 0.0,
        theta_4: 0.0,
      },
      vertices: [
        new Vector3(0, 0, 0),
        new Vector3(0, 0, 0),
        new Vector3(0, 0, 0),
        new Vector3(0, 0, 0),
        new Vector3(0, 0, 0),
        new Vector3(0, 0, 0),
      ],
    });

  // Keep track whether or not we're loading.
  const [isLoading, setIsLoading] = React.useState<boolean>(true);

  // Listen for the arm state changed event.
  useListen<IArmStateChangedEvent>(
    "arm:state-changed",
    (event: IArmStateChangedEvent): void => {
      setVariableArmState({
        kinematicState: event.kinematicState,
        vertices: event.vertices.map(([x, y, z]) => new Vector3(x, y, z)),
      });
    }
  );

  // Perform the initial loading.
  React.useEffect((): (() => void) => {
    let cancelled: boolean = false;

    (async (): Promise<void> => {
      // Sleep for cool effects.
      await timeout(500);

      // Get the kinematic state.
      const getKinematicStateResponse: IGetKinematicStateResponse =
        await invoke<IGetKinematicStateResponse>("get_kinematic_state", {
          command: {},
        });
      if (cancelled) return;

      // Get the kinematic parameters.
      const kinematicParameters: IKinematicParameters =
        await invoke<IKinematicParameters>("get_kinematic_parameters", {
          command: {},
        });
      if (cancelled) return;

      // Get the vertices.
      const getVerticesResponse: IGetVerticesResponse =
        await invoke<IGetVerticesResponse>("get_vertices", { command: {} });
      if (cancelled) return;

      // Set the kinematic parameters.
      setKinematicParameters(kinematicParameters);

      // Set the arm state.
      setVariableArmState({
        kinematicState: getKinematicStateResponse.kinematicState,
        vertices: getVerticesResponse.vertices.map(
          ([x, y, z]) => new Vector3(x, y, z)
        ),
      });

      // Set that we're loading to false.
      setIsLoading(false);
    })();

    return (): void => {
      cancelled = true;
    };
  }, [setKinematicParameters, setVariableArmState, setIsLoading]);

  return (
    <ArmContext.Provider
      value={{
        vertices,
        kinematicState,
        kinematicParameters,
        isLoading,
      }}
    >
      {children}
    </ArmContext.Provider>
  );
};
