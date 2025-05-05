export default class Message {
    constructor(cmd, clock, message) {
        if (typeof cmd === 'string') {
            this.code = Message.get_code(cmd);
        } else {
            this.code = cmd;
        }
        this.clock = BigInt(clock);
        this.data = message;    
        this.length = BigInt(message.length);
    }

    get_command() {
        switch (this.code) {
            case 0:
                return "Acknowledge";
            case 1:
                return "Error";
            case 2:
                return "Update";
            case 3:
                return "StatefulQuery";
            case 4:
                return "StatelessQuery";
            default:
                return "Unknown";
        }
    }

    static get_code(command) {
        switch (command) {
            case "Acknowledge":
                return 0;
            case "Error":
                return 1;
            case "Update":
                return 2;
            case "StatefulQuery":
                return 3;
            case "StatelessQuery":
                return 4;
            default:
                return 255;
        }
    }


    // Convert the object into a Buffer
    serialize() {
        const codeBuffer = Buffer.alloc(1);
        codeBuffer.writeUInt8(this.code, 0);

        const clockBuffer = Buffer.alloc(8);
        clockBuffer.writeBigUInt64BE(this.clock, 0);

        const messageBuffer = Buffer.from(this.data);
        const messageLengthBuffer = Buffer.alloc(8);
        messageLengthBuffer.writeBigUInt64BE(this.length, 0);

        return Buffer.concat([codeBuffer, clockBuffer, messageLengthBuffer, messageBuffer]);
    }

    // Static method to parse a Buffer into a Message object
    static deserialize(buffer) {
        let offset = 0;

        const code = buffer.readUInt8(offset);
        offset += 1;

        const clock = buffer.readBigUInt64BE(offset);
        offset += 8;

        const messageLength = buffer.readBigUInt64BE(offset);
        offset += 8;

        const data = buffer.slice(offset, offset + Number(messageLength));

        return new Message(code, clock, data);
    }
}
