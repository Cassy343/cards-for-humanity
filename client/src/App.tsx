// import './App.css';

import { useEffect, useReducer, useState } from "react";
import { Session, SessionContext } from "./Context";
import { WebsocketBuilder } from 'websocket-ts';
import { Route, Routes } from "react-router";
import { BrowserRouter } from "react-router-dom";
import Homepage from "./homepage/Homepage";
import Lobby from "./lobby/Lobby";

type SessionAction = {
    type: 'socket' | 'username',
    payload: any
};

type SessionReducer = (session: Session, action: SessionAction) => Session;

const sessionReducer: SessionReducer = (session, action) => {
    switch (action.type) {
        case 'socket':
            return {
                ...session,
                socket: action.payload
            }
        case 'username':
            return {
                ...session,
                username: action.payload
            }
    }
};

const connectWs = (session: Session, dispatch: Function) => {
    new WebsocketBuilder(`ws://${window.location.host}/ws`)
        .onOpen((ws, e) => {
            dispatch({
                type: 'socket',
                payload: ws
            });

            console.log("WS Connected");
        })
        .onClose((ws, e) => {
            dispatch({
                type: 'socket',
                payload: null
            });

            console.log("WS Disconnected");
        })
        .build();
}

const App = () => {
    const [session, dispatch] = useReducer(sessionReducer, {
        socket: null,
        username: ''
    });

    useEffect(() => connectWs(session, dispatch), []);

    const sendMessage = (message: any) => {
        if (session.socket) {
            session.socket.send(JSON.stringify(message));
        }
    };

    const setUsername = (username: string) => {
        dispatch({
            type: 'username',
            payload: username
        });
        sendMessage({
            'SetUsername': {
                'username': username
            }
        });
    };

    return (
        <SessionContext.Provider value={session}>
            <BrowserRouter>
                <Routes>
                    <Route
                        path='/'
                        element={<Homepage setUsername={setUsername} />}
                    />
                    <Route
                        path='/lobby'
                        element={<Lobby />}
                    />
                </Routes>
            </BrowserRouter>
        </SessionContext.Provider>
    );
}

export default App;
