import serial
import time

if __name__ == '__main__':
    ser = serial.Serial("/dev/ttyACM0", 115200)
    idx = 100
    while True:
        if ser.in_waiting:
            data = ser.read(ser.in_waiting)
            print(data.decode(), end="")
        time.sleep(.05)


