# Cube Parser

[![Build Status][github-actions-badge]][github-actions]

A program to extract hardware configuration information from the MCU database
files shipped with STM32CubeMX.


## Usage

    cargo run features STM32L0 -d /path/to/stm32cubemx/db/mcu/
    cargo run pin_mappings STM32L0 -d /path/to/stm32cubemx/db/mcu/

Under a default Windows install, the database path is `C:\Program Files
(x86)\STMicroelectronics\STM32Cube\STM32CubeMX\db\mcu`, adjust as appropriate
for your local config. The MCU family name should match one of the MCU families
as defined in `families.xml`. At the time of writing, the following families
are available:

* STM32F0
* STM32F1
* STM32F2
* STM32F3
* STM32F4
* STM32F7
* STM32G0
* STM32G4
* STM32H7
* STM32L0
* STM32L1
* STM32L4
* STM32L4+
* STM32L5
* STM32MP1
* STM32WB


## The STM32CubeMX Database

The STM32CubeMX database contains the following files that are relevant to us:

### Families

In the root, there is a file called `families.xml`. It contains all MCUs
grouped by family (e.g. "STM32F0") and subfamily (e.g. "STM32F0x0 Value Line").

```xml
<Families xsi:noNamespaceSchemaLocation="families.xsd" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
    <Family Name="STM32F0">
        <SubFamily Name="STM32F0x0 Value Line">
            <Mcu Name="STM32F030C6Tx" PackageName="LQFP48" RefName="STM32F030C6Tx" RPN="STM32F030C6">
                <Core>Arm Cortex-M0</Core>
                <Frequency>48</Frequency>
                <Ram>4</Ram>
                ...
            </Mcu>
            <Mcu Name="STM32F030C8Tx" PackageName="LQFP48" RefName="STM32F030C8Tx" RPN="STM32F030C8">
                ...
```

### MCU

Next to the `families.xml` file, there are a lot of MCU definitions. The
filenames match the `Name` attribute in the `Mcu` element above.

For example, the `STM32L071KB(B-Z)Tx.xml` file starts like this:

```xml
<Mcu ClockTree="STM32L0" DBVersion="V3.0" Family="STM32L0" HasPowerPad="false"
        IOType="" Line="STM32L0x1" Package="LQFP32" RefName="STM32L071K(B-Z)Tx"
        xmlns="http://mcd.rou.st.com/modules.php?name=mcu">
	<Core>Arm Cortex-M0+</Core>
	<Frequency>32</Frequency>
	<E2prom>6144</E2prom>
	<Ram>20</Ram>
	<IONb>25</IONb>
	<Die>DIE447</Die>
	<Flash>128</Flash>
	<Flash>192</Flash>
	<Voltage Max="3.6" Min="1.65"/>
	<Current Lowest="0.29" Run="87"/>
	<Temperature Max="125" Min="-40"/>
    ...
```

This first part describes the MCU: How much RAM it has, what the frequency is,
how many I/Os it has, what flash variants there are, etc. Many of these things
are also encoded in the full MCU `RefName`.

Following this general description, there are a number of `IP` elements. IP
stands for "Internal Peripheral". Here we have things like the USART peripherals:

```xml
<IP InstanceName="USART1" Name="USART" Version="sci3_v1_1_Cube"/>
<IP InstanceName="USART2" Name="USART" Version="sci3_v1_1_Cube"/>
```

...or the RCC peripheral:

```xml
<IP InstanceName="RCC" Name="RCC" Version="STM32L051_rcc_v1_0"/>
```

...and, most important to us, the GPIO peripheral.

```xml
<IP ConfigFile="GPIO-STM32L0xx" InstanceName="GPIO" Name="GPIO" Version="STM32L071_gpio_v1_0"/>
```

Here, the value of the `Version` attribute points to the actual GPIO signal
definition.

(There are also some other interesting entries in that file, for example the
mapping from physical pins to internal pin names. Those are not relevant for us
though.)

### GPIO Internal Peripheral

The GPIO IP can be found in the `IP` directory. The relevant file for us is at
`IP/GPIO-<version>_Modes.xml`. The version can be extracted from the `IP`
element shown in the previous section.

In the case of the `STM32L071KBTx`, the relevant GPIO IP version file is at
`IP/GPIO-STM32L071_gpio_v1_0_Modes.xml`.

That file starts out with some `RefParameter` and `RefMode` elements. Relevant
to us are mostly the `GPIO_Pin` elements. They look like this:

```xml
<GPIO_Pin PortName="PB" Name="PB6">
    <SpecificParameter Name="GPIO_Pin">
        <PossibleValue>GPIO_PIN_6</PossibleValue>
    </SpecificParameter>
    <PinSignal Name="I2C1_SCL">
        <SpecificParameter Name="GPIO_AF">
            <PossibleValue>GPIO_AF1_I2C1</PossibleValue>
        </SpecificParameter>
    </PinSignal>
    <PinSignal Name="LPTIM1_ETR">
        <SpecificParameter Name="GPIO_AF">
            <PossibleValue>GPIO_AF2_LPTIM1</PossibleValue>
        </SpecificParameter>
    </PinSignal>
    <PinSignal Name="TSC_G5_IO3">
        <SpecificParameter Name="GPIO_AF">
            <PossibleValue>GPIO_AF3_TSC</PossibleValue>
        </SpecificParameter>
    </PinSignal>
    <PinSignal Name="USART1_TX">
        <SpecificParameter Name="GPIO_AF">
            <PossibleValue>GPIO_AF0_USART1</PossibleValue>
        </SpecificParameter>
    </PinSignal>
</GPIO_Pin>
```

As you can see, this element describes the pin `PB6`. Depending on the chosen
Alternative Function (AF), it can become an `I2C1_SCL` pin (AF1), a `USART1_TX`
pin (AF0), or some other variants.


## GPIO Feature Groups

When generating pin function mappings, we want to avoid generating a mapping
for every possible MCU, since that would result in dozens or even hundreds of
pin definitions. However, if we don't want a mapping per MCU, we need to group
them somehow. The best way for grouping is probably to follow ST's grouping,
which is encoded in the IP versions described above.

The feature names are mapped as follows:

- `STM32L031_gpio_v1_0` -> `io-STM32L031`
- `STM32L051_gpio_v1_0` -> `io-STM32L051`
- `STM32L152x8_gpio_v1_0` -> `io-STM32L152x8`

For example, the GPIO IP file named "STM32L031_gpio_v1_0" is shared among the
following MCUs:

- STM32L010C6Tx
- STM32L031C(4-6)Tx
- STM32L031C(4-6)Tx
- STM32L031C6Ux
- STM32L031E(4-6)Yx
- STM32L031E(4-6)Yx
- STM32L031F(4-6)Px
- STM32L031F(4-6)Px
- STM32L031G(4-6)Ux
- STM32L031G(4-6)Ux
- STM32L031G6UxS
- STM32L031K(4-6)Tx
- STM32L031K(4-6)Tx
- STM32L031K(4-6)Ux
- STM32L031K(4-6)Ux
- STM32L041C(4-6)Tx
- STM32L041C(4-6)Tx
- STM32L041E6Yx
- STM32L041F6Px
- STM32L041G6Ux
- STM32L041G6UxS
- STM32L041K6Tx
- STM32L041K6Ux

As you can see, this may be a bit confusing due to the fact that both the
`STM32L010C6Tx` and the `STM32L041E6Yx` require the `io-STM32L031` feature.
However, sticking to the (sometimes non-logical) grouping used in the CubeMX
database is probably still better than creating our own grouping, which may be
broken at any time by ST releasing a new MCU in a pre-existing group, but with
a different, incompatible GPIO IP version.

In order to simplify the GPIO IP version selection for the user, alias features
are generated. These are purely a convenience for the user and are never used
directly as feature gates in the source code.


<!-- Badges -->
[github-actions]: https://github.com/dbrgn/cube-parse/actions?query=branch%3Amaster
[github-actions-badge]: https://github.com/dbrgn/cube-parse/workflows/CI/badge.svg
