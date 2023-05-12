import { Context, createContext } from "react";
import { useWebSocket } from "react-use-websocket/dist/lib/use-websocket";

export type Ids = {
    gameId: number,
    playerId: number
};

export type Session = {
    username: string,
    ids: Ids | null
};

export type CfhSend = (msg: string, payload: object) => void;

export type CfhWs = {
    sendMessage: CfhSend,
    lastMessage: any
};

export const useCfhWs = (filter?: (msg: any) => boolean): CfhWs => {
    let { sendJsonMessage, lastJsonMessage } = useWebSocket(`ws://${window.location.host}/ws`, {
        share: true,
        filter: msg => {
            let json;
            try {
                json = JSON.parse(msg.data);
            } catch (_) {
                return false;
            }

            if (typeof json.msg !== 'string') {
                return false;
            }

            return filter ? filter(json) : true;
        }
    });

    return {
        sendMessage: (msg, payload) => {
            sendJsonMessage({
                ...payload,
                msg: msg
            });
        },
        lastMessage: lastJsonMessage
    }
};

// @ts-ignore
export const SessionContext: Context<Session> = createContext(undefined);