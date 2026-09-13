import os

from pysolarmanv5 import PySolarmanV5

HOST = os.environ["SOLARMAN_HOST"]
LOGGER_SERIAL = int(os.environ["SOLARMAN_LOGGER_SERIAL"])

inverter = PySolarmanV5(
    HOST,
    LOGGER_SERIAL,
    port=8899,
    mb_slave_id=1,
    verbose=True,
)

tests = [
    ("État",          0x0404, 1),
    ("Réseau",        0x0484, 2),
    ("PV1 + PV2",     0x0584, 6),
    ("Puissance PV",  0x05C4, 1),
    ("Batterie",      0x0604, 6),
]

try:
    for name, address, quantity in tests:
        try:
            values = inverter.read_holding_registers(
                register_addr=address,
                quantity=quantity,
            )
            print(f"{name:15} {address:#06x} : {values}")
        except Exception as error:
            print(f"{name:15} {address:#06x} : ERREUR {error}")
finally:
    inverter.disconnect()
