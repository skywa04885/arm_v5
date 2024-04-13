import {
  Button,
  Card,
  CardBody,
  CardFooter,
  CardHeader,
  Collapse,
  FormControl,
  FormLabel,
  Heading,
  Icon,
  InputGroup,
  InputLeftAddon,
  NumberDecrementStepper,
  NumberIncrementStepper,
  NumberInput,
  NumberInputField,
  NumberInputStepper,
  Select,
  Stack,
  Switch,
  Text,
  useDisclosure,
} from "@chakra-ui/react";
import React, { ChangeEvent } from "react";
import { useArm } from "../providers/ArmProvider";
import { ChevronUpDownIcon, PlayCircleIcon } from "@heroicons/react/16/solid";
import {
  EControlMode,
  ITranslateEndEffectorControlMode,
  TControlMode,
} from "../App";
import { Vector3 } from "three";

export interface ITranslateEndEffectorComponentProps {
  controlMode: ITranslateEndEffectorControlMode;
  setControlMode: React.Dispatch<React.SetStateAction<TControlMode>>;
}

const TranslateEndEffectorComponent = ({
  controlMode,
  setControlMode,
}: ITranslateEndEffectorComponentProps): React.ReactElement => {
  const { vertices } = useArm();

  const onChangeX = (_: string, x: number) => {
    const { y, z } = controlMode.target_position;

    setControlMode({
      ...controlMode,
      target_position: new Vector3(x, y, z),
    });
  };

  const onChangeY = (_: string, y: number) => {
    const { x, z } = controlMode.target_position;

    setControlMode({
      ...controlMode,
      target_position: new Vector3(x, y, z),
    });
  };

  const onChangeZ = (_: string, z: number) => {
    const { x, y } = controlMode.target_position;

    setControlMode({
      ...controlMode,
      target_position: new Vector3(x, y, z),
    });
  };

  return (
    <Stack direction={"column"}>
      {/* X coordinate */}
      <FormControl>
        <InputGroup size={"sm"}>
          <InputLeftAddon>
            <Text color={"red"} fontWeight={"bold"}>
              x
            </Text>
          </InputLeftAddon>
          <NumberInput
            width={"full"}
            value={controlMode.target_position.x}
            onChange={onChangeX}
            precision={2}
          >
            <NumberInputField />
            <NumberInputStepper>
              <NumberIncrementStepper />
              <NumberDecrementStepper />
            </NumberInputStepper>
          </NumberInput>
        </InputGroup>
      </FormControl>
      {/* Y coordinate */}
      <FormControl>
        <InputGroup size={"sm"}>
          <InputLeftAddon>
            <Text color={"green"} fontWeight={"bold"}>
              y
            </Text>
          </InputLeftAddon>
          <NumberInput
            width={"full"}
            value={controlMode.target_position.y}
            onChange={onChangeY}
            precision={2}
          >
            <NumberInputField />
            <NumberInputStepper>
              <NumberIncrementStepper />
              <NumberDecrementStepper />
            </NumberInputStepper>
          </NumberInput>
        </InputGroup>
      </FormControl>
      {/* Z coordinat */}
      <FormControl>
        <InputGroup size={"sm"}>
          <InputLeftAddon>
            <Text color={"blue"} fontWeight={"bold"}>
              z
            </Text>
          </InputLeftAddon>
          <NumberInput
            width={"full"}
            value={controlMode.target_position.z}
            onChange={onChangeZ}
            precision={2}
          >
            <NumberInputField />
            <NumberInputStepper>
              <NumberIncrementStepper />
              <NumberDecrementStepper />
            </NumberInputStepper>
          </NumberInput>
        </InputGroup>
      </FormControl>
    </Stack>
  );
};

export interface IManualControlComponentProps {
  controlMode: TControlMode;
  setControlMode: React.Dispatch<React.SetStateAction<TControlMode>>;
}

export const ManualControlComponent = ({
  controlMode,
  setControlMode,
}: IManualControlComponentProps): React.ReactElement => {
  const { isOpen, onToggle: toggleIsOpen } = useDisclosure();

  const renderModeOptions = React.useCallback((): React.ReactElement[] => {
    return Object.values(EControlMode).map(
      (value: EControlMode): React.ReactElement => {
        return <option key={value}>{value}</option>;
      }
    );
  }, []);

  const renderControlModeOptions = React.useCallback((): React.ReactElement => {
    switch (controlMode.mode) {
      case EControlMode.RotateEndEffector:
        return <></>;
      case EControlMode.TranslateEndEffector:
        return (
          <TranslateEndEffectorComponent
            setControlMode={setControlMode}
            controlMode={controlMode as ITranslateEndEffectorControlMode}
          />
        );
    }
  }, [controlMode, setControlMode]);

  return (
    <Card size={"sm"} width={96} variant={"outline"}>
      <CardHeader
        userSelect={"none"}
        cursor={"pointer"}
        onClick={() => toggleIsOpen()}
      >
        <Heading size={"sm"}>Control</Heading>
      </CardHeader>
      <Collapse in={isOpen}>
        <CardBody>
          <Stack direction={"column"}>
            {/* Mode selection */}
            <FormControl>
              <FormLabel>Control mode</FormLabel>
              <Select
                icon={
                  <Icon>
                    <ChevronUpDownIcon />
                  </Icon>
                }
                value={controlMode.mode}
                // onChange={(e: ChangeEvent<HTMLSelectElement>): void =>
                //   setControl(e.target.value as ManualControlMode)
                // }
              >
                {renderModeOptions()}
              </Select>
            </FormControl>
            {/* Preview enabling/ disabling */}
            <Stack direction={"row"} alignItems={"center"}>
              <Switch />
              <Text>Preview</Text>
            </Stack>
          </Stack>
        </CardBody>
        <CardBody>{renderControlModeOptions()}</CardBody>
        <CardFooter>
          <Button
            leftIcon={
              <Icon>
                <PlayCircleIcon />
              </Icon>
            }
            size={"sm"}
            variant={"solid"}
            colorScheme={"gray"}
          >
            Begin motion
          </Button>
        </CardFooter>
      </Collapse>
    </Card>
  );
};
