export default class MessageProcessor {
  constructor() {
    this.counterQueue = [];
    this.messageBuffer = new Map();
  }

  enqueueCounter(counter) {
    this.counterQueue.push(counter);
  }

  processMessage(message) {
    if (this.counterQueue.length === 0) {
      console.warn("No expected counter in queue");
      return;
    }

    if (this.counterQueue[0] === message.clock) {
      console.log("Processing message:", message);
      this.counterQueue.shift();

      while (this.messageBuffer.has(this.counterQueue[0])) {
        const nextMessage = this.messageBuffer.get(this.counterQueue[0]);
        this.messageBuffer.delete(this.counterQueue[0]);
        this.processMessage(nextMessage, this.counterQueue[0]);
      }
    } else {
      console.log("Storing message for later:", message);
      this.messageBuffer.set(message.clock, message);
    }
  }
}
