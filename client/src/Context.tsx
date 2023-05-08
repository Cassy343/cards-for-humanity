import { Context, createContext } from "react";
import { Websocket } from "websocket-ts/lib";

export type Session = {
    socket: Websocket | null,
    username: string
};

// @ts-ignore
export const SessionContext: Context<Session> = createContext(undefined);