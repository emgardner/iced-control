import serial
import time

if __name__ == '__main__':
    ser = serial.Serial("/dev/ttyACM0", 115200) 
    set_state = True
    t = time.time()
    #while 1:
    #    print("SENT", set_state)
    #    if set_state == True:
    #        ser.write("P\n".encode())
    #    else:
    #        ser.write("C\n".encode())
    #    while not ser.in_waiting:
    #        pass
    #    rcv = ser.readline()
    #    print("RECEIVED: ", time.time() - t, rcv.decode(), end="")
    #    t = time.time()
    #    set_state = not set_state
    #    time.sleep(.10)

    #while 1:
    #    print("SENT", set_state)
    #    if set_state == True:
    #        ser.write("E\n".encode())
    #    else:
    #        ser.write("O\n".encode())
    #    while not ser.in_waiting:
    #        pass
    #    rcv = ser.readline()
    #    print("RECEIVED: ", time.time() - t, rcv.decode(), end="")
    #    t = time.time()
    #    set_state = not set_state
    #    time.sleep(2)
    i = 10
    while 1:
        print("SENT", set_state)
        ser.write(f"D{i}\n".encode())
        while not ser.in_waiting:
            pass
        rcv = ser.readline()
        print("RECEIVED: ", time.time() - t, rcv.decode(), end="")
        i += 10
        if i > 100:
            i = 10
        t = time.time()
        set_state = not set_state
        time.sleep(2)
    """
    f = 1000
    while 1:
        print("SENT", set_state)
        ser.write(f"F{f}\n".encode())
        while not ser.in_waiting:
            pass
        rcv = ser.readline()
        print("RECEIVED: ", time.time() - t, rcv.decode(), end="")
        f += 100
        if f > 10000:
            f = 100
        t = time.time()
        set_state = not set_state
        #time.sleep(2)
    """
