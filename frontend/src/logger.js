import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { dirname } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

export class Logger {
    constructor(fileName) {
        this.logFile = path.join(__dirname, fileName);

        const entry = 'req_id, clock, op, elapsed\n';
        fs.writeFile(this.logFile, entry, (err) => {
            if (err) console.error('Failed to write log:', err);
        });
    }

    log(id, clock, op, elapsed) {
        const entry = `${id}, ${clock}, ${op},${elapsed}\n`;
        fs.appendFile(this.logFile, entry, (err) => {
            if (err) console.error('Failed to write log:', err);
        });
    }
}

