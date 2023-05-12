import { Box, Button, ButtonGroup, Card, FormControl, FormHelperText, Input, Stack, Typography } from '@mui/material';
import './Lobby.css';
import { useEffect, useState } from 'react';
import { CfhSend, useCfhWs } from '../Session';
import { Navigate } from 'react-router';

const joinGame = (sendMessage: CfhSend, gameId: string) => {
    const numericId = parseInt(gameId, 16);
    sendMessage('JoinGame', {
        id: numericId
    });
};

const createGame = (sendMessage: CfhSend) => {
    sendMessage('CreateGame', {});
};

type LobbyProps = {
    setIds: (game_id: number, player_id: number) => void
};

const Lobby = (props: LobbyProps) => {
    const { sendMessage, lastMessage } = useCfhWs(msg => msg.msg === 'JoinResponse');
    const gameIdRegex = /^[0-9a-fA-F]{4}$/g;
    const [gameId, setGameId] = useState('');
    const gameIdValid = gameIdRegex.test(gameId);

    let errorText = null;
    let game_id: number | null = null;
    let player_id: number | null = null;

    if (lastMessage && lastMessage.msg === 'JoinResponse') {
        if (lastMessage.response === null) {
            errorText = 'Game not found';
        } else if (lastMessage.response.type === 'Rejected') {
            errorText = 'Game is currently full';
        } else {
            game_id = lastMessage.response.game_id;
            player_id = lastMessage.response.player_id;
        }
    }

    if (!(gameIdValid || gameId === '')) {
        errorText = 'Game ID should be four characters 0-9 or A-F';
    }

    useEffect(() => {
        if (player_id !== null && game_id !== null) {
            props.setIds(game_id, player_id);
        }
    }, [props, game_id, player_id]);

    if (player_id !== null && game_id !== null) {
        return (<>
            <Navigate to='/game' />
        </>)
    }

    return (<>
        <Box id='create-join-container'>
            <Card id='create-join-card'>
                <Stack
                    id='create-join-form'
                    spacing={2}
                >
                    <Typography>Join a game by entering its ID or create a new game.</Typography>
                    <FormControl variant='standard'>
                        <Input
                            placeholder='D332'
                            error={errorText !== null}
                            onChange={newValue => setGameId(newValue.target.value)}
                        />
                        {
                            errorText
                                ? <FormHelperText error>
                                    {errorText}
                                </FormHelperText>
                                : null
                        }
                    </FormControl>
                    <ButtonGroup variant='outlined'>
                        <Button
                            disabled={!gameIdValid}
                            onClick={_ => joinGame(sendMessage, gameId)}
                            sx={{ width: '50%' }}
                        >Join Game</Button>
                        <Button
                            onClick={_ => createGame(sendMessage)}
                            sx={{ width: '50%' }}
                        >Create Game</Button>
                    </ButtonGroup>
                </Stack>
            </Card>
        </Box>
    </>)
};

export default Lobby;