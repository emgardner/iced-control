# Rust Iced GUI Demo for controlling and STM32 MCU

This is a simple demo illustrating a few different things that I learned along the way making my first iced gui. This is very much still a WIP and will be updated to add capabilities to both the device and gui.

The repositories are as follows:
- iced-gui 
  - GUI built using iced
- iced-driver
  - Serial driver for the program running on the MCU. This leverages the *tokio_serial* library
- iced-mcu
  - A program running on a NUCLEO-L476RG that takes serial commands


## The GUI consists of just two pages:

### A page for opening a com port

![Open Page](./images/open.png?raw=true "Open Page")

### A page for controlling a microcontroller

![Control Page](./images/control.png?raw=true "Control Page")


