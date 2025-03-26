
const isDev = process.env.NODE_ENV === 'dev';

export default class MessageProcessor {
    constructor() {
        this.counterQueue = [];
        this.messageBuffer = new Map();
    }

    enqueueCounter(counter, callback) {
        this.counterQueue.push( { clock: counter, callback: callback } );
    }

    processMessage(message) {
        if (this.counterQueue.length === 0) {
            console.warn("No expected counter in queue");
            return;
        }

        if (this.counterQueue[0].clock === message.clock) {
            if(isDev){
                console.log("Processing message:", message.clock);
            }
            this.counterQueue[0].callback(message.data);

            this.counterQueue.shift();
            while (this.counterQueue.length > 0 && this.messageBuffer.has(this.counterQueue[0].clock)) {
                const nextMessage = this.messageBuffer.get(this.counterQueue[0].clock);
                this.messageBuffer.delete(this.counterQueue[0].clock);
                this.processMessage(nextMessage, this.counterQueue[0].clock);
            }
        } else {
            if(isDev){
                console.log("Storing message for later:", message.clock);
            }
            this.messageBuffer.set(message.clock, message);
        }
    }
}
