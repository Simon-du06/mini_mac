fn modbus_crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;

    for &byte in data {
        crc ^= u16::from(byte);
        for _ in 0..8 {
            let last_bit_is_one = crc & 1 != 0;
            crc >>= 1;
            if last_bit_is_one {
                crc ^= 0xA001;
            }
        }
    }
    crc
}

pub fn build_modbus_read_request(
    slave_id: u8,
    register: u16,
    quantity: u16,
) -> Vec<u8> {
    let mut request: Vec<u8> = vec![];
    const FUNC_CODE: u8 = 0x03;

    request.push(slave_id);
    request.push(FUNC_CODE);
    let register_vec = register.to_be_bytes();
    request.extend_from_slice(&register_vec);
    let quantity_vec = quantity.to_be_bytes();
    request.extend_from_slice(&quantity_vec);
    
    let crc = modbus_crc16(&request);
    let crc_le = crc.to_le_bytes();
    request.extend_from_slice(&crc_le);

    request
}

fn build_solarman_payload(modbus_request: &[u8]) -> Vec<u8> {
    let mut payload: Vec<u8> = vec![];

    payload.push(0x02);
    for _ in 0..14 {
        payload.push(0x00);
    }
    payload.extend_from_slice(modbus_request);
    payload
}

fn build_solarman_frame(
    logger_serial: u32,
    sequence: u16,
    payload: &[u8],
) -> Vec<u8> {
    let mut head: Vec<u8> = vec![];

    //init frame
    head.push(0xA5);

    //payload len
    let lenght = u16::try_from(payload.len()).expect("payload length exceeds u16");
    head.extend_from_slice(&lenght.to_le_bytes());

    //control code to read
    head.push(0x10);
    head.push(0x45);

    //sequeznce reference
    head.extend_from_slice(&sequence.to_le_bytes());

    //logger sn
    head.extend_from_slice(&logger_serial.to_le_bytes());

    //add the payload next to the request
    head.extend_from_slice(payload);

    //checksum
    head.push(solarman_checksum(&head));

    head.push(0x15);

    head
}

fn solarman_checksum(frame: &[u8]) -> u8 {
    let mut checksum: u8 = 0;

    for i in 1..frame.len() {
        checksum = checksum.wrapping_add(frame[i]);
    }
    checksum
}


#[test]
fn crc_matches_real_solarman_request() {
    let request = [0x01, 0x03, 0x04, 0x04, 0x00, 0x01];

    let crc = modbus_crc16(&request);

    assert_eq!(crc, 0xFBC4);
}

#[test]
fn modbus_request_match () {
    let request = build_modbus_read_request(1, 0x0404, 1);

    assert_eq!(request, [0x01, 0x03, 0x04, 0x04, 0x00, 0x01, 0xC4, 0xFB]);
}

#[test]
fn solarman_payload_wraps_modbus_request() {
    let modbus_request = build_modbus_read_request(1, 0x0404, 1);
    let payload = build_solarman_payload(&modbus_request);

    assert_eq!(payload.len(), 23);
    assert_eq!(payload[0], 0x02);
    assert!(payload[1..15].iter().all(|&byte| byte == 0));
    assert_eq!(&payload[15..], modbus_request.as_slice());
}

#[test]
fn solarman_frame_has_expected_header() {
    let modbus_request = build_modbus_read_request(1, 0x0404, 1);
    let payload = build_solarman_payload(&modbus_request);
    let frame = build_solarman_frame(0x01020304, 1, &payload);

    assert_eq!(frame.len(), 36);
    assert_eq!(&frame[..11], [
        0xA5, 0x17, 0x00, 0x10, 0x45, 0x01, 0x00, 0x04, 0x03, 0x02, 0x01,
    ]);
    assert_eq!(&frame[11..34], payload.as_slice());
}

#[test]
fn solarman_checksum_matches_reference_frame() {
    let modbus_request = build_modbus_read_request(1, 0x0404, 1);
    let payload = build_solarman_payload(&modbus_request);
    let frame = build_solarman_frame(0x01020304, 1, &payload);

    assert_eq!(solarman_checksum(&frame[..34]), 0x45);
}

#[test]
fn solarman_frame_has_checksum_and_end_marker() {
    let modbus_request = build_modbus_read_request(1, 0x0404, 1);
    let payload = build_solarman_payload(&modbus_request);
    let frame = build_solarman_frame(0x01020304, 1, &payload);

    assert_eq!(frame[34], 0x45);
    assert_eq!(frame[35], 0x15);
}
