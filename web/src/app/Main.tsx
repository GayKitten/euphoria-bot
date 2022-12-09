// create a guest main page & a user main page

import { Button, Container, Paper, Typography } from '@mui/material';
import { BridgeContext } from 'App';
import { useMeQuery, User } from 'features/api-slice';
import React, { useContext } from 'react';

const GuestMain = () => {
	return (<div/>);
};

const UserMain: React.FC<{user: User}> = ({user}) => {
	
	const {bridge, connect} = useContext(BridgeContext);

	return (
		<Container sx={{mt: 4}}>
			<Paper sx={{p: 4}}>
				<Typography>
					Connection: {bridge !== null ? 'Connected' : 'Disconnected'}
				</Typography>
					{bridge !== null || (
						<Button
							variant='outlined'
							color='success'
							onClick={() => connect("ws://localhost:12345")}
						>
							Connect
						</Button>
					)}
			</Paper>
		</Container>
	);
};

const Main = () => {
	const { data } = useMeQuery();

	if (data !== undefined && data.status === 'LoggedIn') return <UserMain user={data.user} />;
	return <GuestMain />;
};

export default Main;
