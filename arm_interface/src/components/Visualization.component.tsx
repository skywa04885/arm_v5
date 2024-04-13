import { Html, OrbitControls, PerspectiveCamera } from "@react-three/drei";
import { Canvas } from "@react-three/fiber";
import React, { useRef } from "react";
import { BufferGeometry, Color, Vector3 } from "three";
import { useArm } from "../providers/ArmProvider";
import { Text } from "@chakra-ui/react";
import { EControlMode, TControlMode } from "../App";

export interface IMoveEndEffectorPreviewComponentProps {
  target: Vector3;
}

export const MoveEndEffectorPreviewComponent = ({
  target,
}: IMoveEndEffectorPreviewComponentProps): React.ReactElement => {
  const { vertices } = useArm();

  const pointsBufferGeometry = React.useRef<BufferGeometry>(null);
  const lineBufferGeometry = React.useRef<BufferGeometry>(null);

  React.useEffect((): void => {
    lineBufferGeometry.current?.setFromPoints([vertices[5], target]);
  }, [vertices, target, lineBufferGeometry]);

  React.useEffect((): void => {
    pointsBufferGeometry.current?.setFromPoints([target]);
  }, [target, pointsBufferGeometry]);

  return (
    <group name="Move end effector preview">
      {/* The movement preview line */}
      <line>
        <bufferGeometry ref={lineBufferGeometry} />
        <lineDashedMaterial
          scale={1}
          linewidth={2}
          transparent
          opacity={0.4}
          color={"red"}
          gapSize={20}
          dashSize={3}
        />
      </line>
      {/* The end point of the preview line */}
      <points>
        <bufferGeometry ref={pointsBufferGeometry} />
        <pointsMaterial size={2} color={"red"} />
      </points>
    </group>
  );
};

export interface IArmComponentAngleProps {
  angle: number;
  position: Vector3;
}

export const ArmComponentAngle = ({
  angle,
  position,
}: IArmComponentAngleProps): React.ReactElement => {
  return (
    <mesh position={position}>
      <Html style={{ pointerEvents: "none" }}>
        <Text fontSize={"small"} color={"white"}>
          {angle.toPrecision(2)}&deg;
        </Text>
      </Html>
    </mesh>
  );
};

export const ArmComponent = (): React.ReactElement => {
  const { vertices, kinematicState } = useArm();

  const armLineRef = useRef<BufferGeometry>(null);
  const armPointsRef = useRef<BufferGeometry>(null);

  React.useEffect((): void => {
    armLineRef.current?.setFromPoints(vertices);
    armPointsRef.current?.setFromPoints(vertices);
  }, [vertices, armLineRef, armPointsRef]);

  return (
    <group name={"The arm"}>
      {/* The actual line */}
      <line>
        <bufferGeometry ref={armLineRef} />
        <lineBasicMaterial
          color={"green"}
          linewidth={2}
          linecap={"round"}
          linejoin={"round"}
          opacity={0.7}
          transparent={true}
        />
      </line>
      {/* The angles */}
      <ArmComponentAngle
        position={vertices[0]}
        angle={kinematicState.theta_0}
      />
      <ArmComponentAngle
        position={vertices[1]}
        angle={kinematicState.theta_1}
      />
      <ArmComponentAngle
        position={vertices[2]}
        angle={kinematicState.theta_2}
      />
      <ArmComponentAngle
        position={vertices[3]}
        angle={kinematicState.theta_3}
      />
      <ArmComponentAngle
        position={vertices[4]}
        angle={kinematicState.theta_4}
      />
      {/* The points of the line */}
      <points>
        <bufferGeometry ref={armPointsRef} />
        <pointsMaterial
          size={1}
          transparent={true}
          opacity={0.1}
          color={"white"}
        />
      </points>
    </group>
  );
};

export interface IVisualizationComponentProps {
  controlMode: TControlMode;
  setControlMode: React.Dispatch<React.SetStateAction<TControlMode>>;
}

export const VisualizationComponent = ({
  controlMode,
  setControlMode,
}: IVisualizationComponentProps): React.ReactElement => {
  const background: Color = new Color("#222222");

  const renderPreviewComponent = React.useCallback((): React.ReactElement => {
    switch (controlMode.mode) {
      case EControlMode.RotateEndEffector:
        return <></>;
      case EControlMode.TranslateEndEffector:
        return <MoveEndEffectorPreviewComponent target={controlMode.target_position} />;
    }
  }, [controlMode]);

  return (
    <Canvas scene={{ background }}>
      <PerspectiveCamera position={[60, 60, 60]} makeDefault={true} />
      <ambientLight />
      <OrbitControls dampingFactor={1} />
      <axesHelper scale={6} />
      <ArmComponent />
      {renderPreviewComponent()}
    </Canvas>
  );
};
