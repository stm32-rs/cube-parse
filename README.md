# Cube Parser

A program to extract AF modes on MCU pins from the database files provided with STM32CubeMX.

## Usage
```
cargo run $PATH_TO_MCU_DB_DIR $NAME_OF_MCU_VARIANT
```
Under a default windows install `$PATH_TO_MCU_DB_DIR` is `C:\Program Files (x86)\STMicroelectronics\STM32Cube\STM32CubeMX\db\mcu`, adjust as appropriate for your local config. The MCU variant name should match the name of one of the files in the indicated folder.