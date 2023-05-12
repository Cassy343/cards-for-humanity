import { Box, Card, FormControl, Input, Stack, Typography } from '@mui/material';
import './Lobby.css';

const Lobby = (props: any) => {
    return (<>
        <Box id='create-join-container'>
            <Card id='create-join-card'>
                <Stack
                    id='create-join-form'
                    spacing={2}
                >
                    <Typography>Join a game by entering its ID or create a new game.</Typography>
                    {/* <FormControl variant='standard'>
                        <Input
                            placeholder='D332'
                            onChange={newValue => setTmpUsername(newValue.target.value)}
                        />
                    </FormControl>
                    <Button
                        variant='outlined'
                        disabled={tmpUsername.length === 0}
                        onClick={_ => props.setUsername(tmpUsername)}
                    >Join Lobby</Button> */}
                </Stack>
            </Card>
        </Box>
    </>)
};

export default Lobby;