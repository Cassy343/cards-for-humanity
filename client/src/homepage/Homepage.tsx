import './Homepage.css';

import { Box, Button, Card, FormControl, Input, Stack, Typography } from "@mui/material";
import { useContext, useRef, useState } from "react";
import { SessionContext } from '../Session';
import { Navigate } from 'react-router';

type HomepageProps = {
    setUsername: (username: string) => void
};

const Homepage = (props: HomepageProps) => {
    const session = useContext(SessionContext);

    if (session.username !== '') {
        return (<>
            <Navigate to='/lobby' />
        </>);
    }

    const [tmpUsername, setTmpUsername] = useState('');

    return (<>
        <Box id='login-container'>
            <Card id='login-card'>
                <Stack
                    id='login-form'
                    spacing={2}
                >
                    <Typography>Pick a username to get started!</Typography>
                    <FormControl variant='standard'>
                        <Input
                            placeholder='username'
                            onChange={newValue => setTmpUsername(newValue.target.value)}
                        />
                    </FormControl>
                    <Button
                        variant='outlined'
                        disabled={tmpUsername.length === 0}
                        onClick={_ => props.setUsername(tmpUsername)}
                    >Join Lobby</Button>
                </Stack>
            </Card>
        </Box>
    </>);
};

export default Homepage;