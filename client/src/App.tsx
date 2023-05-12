// import './App.css';

import { useEffect, useReducer, useState } from "react";
import { Session, SessionContext, useCfhWs } from "./Session";
import { WebsocketBuilder } from 'websocket-ts';
import { Route, Routes } from "react-router";
import { BrowserRouter } from "react-router-dom";
import Homepage from "./homepage/Homepage";
import Lobby from "./lobby/Lobby";
import Game from "./game/Game";

type SessionAction = {
    type: 'setUsername' | 'setIds',
    payload: any
};

type SessionReducer = (session: Session, action: SessionAction) => Session;

const sessionReducer: SessionReducer = (session, action) => {
    switch (action.type) {
        case 'setUsername':
            return {
                ...session,
                username: action.payload
            }
        case 'setIds':
            console.log(`Game ID: ${action.payload.game_id}, Player ID: ${action.payload.player_id}`);
            return {
                ...session,
                ids: action.payload
            }
    }
};

const App = () => {
    const [session, dispatch] = useReducer(sessionReducer, {
        username: '',
        ids: null
    });
    const { sendMessage } = useCfhWs(_ => false);

    const setUsername = (username: string) => {
        dispatch({
            type: 'setUsername',
            payload: username
        });
        sendMessage('SetUsername', {
            'username': username
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
                        element={<Lobby setIds={(game_id, player_id) => dispatch({
                            type: 'setIds',
                            payload: {
                                game_id: game_id,
                                player_id: player_id
                            }
                        })} />}
                    />
                    <Route
                        path='/game'
                        element={<Game />}
                    />
                </Routes>
            </BrowserRouter>
        </SessionContext.Provider>
    );
}

export default App;
