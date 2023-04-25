"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g;
    return g = { next: verb(0), "throw": verb(1), "return": verb(2) }, typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (g && (g = 0, op[0] && (_ = 0)), _) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.waitForThreadExec = exports.verifyAmount = exports.stream_program_logs = exports.print_tx = exports.print_thread = exports.print_address = void 0;
var chai_1 = require("chai");
var child_process_1 = require("child_process");
var spl_token_1 = require("@solana/spl-token");
//  import { ClockworkProvider, PAYER_PUBKEY } from "@clockwork-xyz/sdk";
var print_address = function (label, address) {
    console.log("".concat(label, ": https://explorer.solana.com/address/").concat(address, "?cluster=devnet"));
};
exports.print_address = print_address;
var print_thread = function (clockworkProvider, address) { return __awaiter(void 0, void 0, void 0, function () {
    var threadAccount;
    return __generator(this, function (_a) {
        switch (_a.label) {
            case 0: return [4 /*yield*/, clockworkProvider.getThreadAccount(address)];
            case 1:
                threadAccount = _a.sent();
                console.log("\nThread: ", threadAccount, "\n");
                print_address("🧵 Thread", address);
                console.log("\n");
                return [2 /*return*/];
        }
    });
}); };
exports.print_thread = print_thread;
var print_tx = function (label, address) {
    console.log("".concat(label, ": https://explorer.solana.com/tx/").concat(address, "?cluster=devnet"));
};
exports.print_tx = print_tx;
var stream_program_logs = function (programId) {
    var cmd = (0, child_process_1.spawn)("solana", ["logs", "-u", "devnet", programId.toString()]);
    cmd.stdout.on("data", function (data) {
        console.log("Program Logs: ".concat(data));
    });
};
exports.stream_program_logs = stream_program_logs;
var verifyAmount = function (connection, ata, expectedAmount) { return __awaiter(void 0, void 0, void 0, function () {
    var amount;
    return __generator(this, function (_a) {
        switch (_a.label) {
            case 0: return [4 /*yield*/, (0, spl_token_1.getAccount)(connection, ata)];
            case 1:
                amount = (_a.sent()).amount;
                chai_1.assert.equal(amount.toString(), expectedAmount.toString());
                return [2 /*return*/, amount];
        }
    });
}); };
exports.verifyAmount = verifyAmount;
var lastThreadExec = BigInt(0);
var waitForThreadExec = function (clockworkProvider, thread, maxWait) {
    if (maxWait === void 0) { maxWait = 60; }
    return __awaiter(void 0, void 0, void 0, function () {
        var i, execContext;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    i = 1;
                    _a.label = 1;
                case 1:
                    if (!true) return [3 /*break*/, 4];
                    return [4 /*yield*/, clockworkProvider.getThreadAccount(thread)];
                case 2:
                    execContext = (_a.sent()).execContext;
                    if (execContext) {
                        if (lastThreadExec.toString() == "0" || execContext.lastExecAt > lastThreadExec) {
                            lastThreadExec = execContext.lastExecAt;
                            return [3 /*break*/, 4];
                        }
                    }
                    if (i == maxWait)
                        throw Error("Timeout");
                    i += 1;
                    return [4 /*yield*/, new Promise(function (r) { return setTimeout(r, i * 1000); })];
                case 3:
                    _a.sent();
                    return [3 /*break*/, 1];
                case 4: return [2 /*return*/];
            }
        });
    });
};
exports.waitForThreadExec = waitForThreadExec;
