# Cube Parser

A program to extract AF modes on MCU pins from the database files provided with STM32CubeMX.

## Usage

```
cargo run $PATH_TO_MCU_DB_DIR $NAME_OF_MCU_FAMILY
```

Under a default windows install `$PATH_TO_MCU_DB_DIR` is `C:\Program Files
(x86)\STMicroelectronics\STM32Cube\STM32CubeMX\db\mcu`, adjust as appropriate
for your local config. The MCU family name should match one of the MCU families
as defined in `families.xml`. The program will output one AF mode definition
per GPIO variant, with cfg feature gates for the different MCU variants that
utilise that GPIO type.
