import { Container, List, ListItem, ListItemText, Paper, Stack, Typography } from "@mui/material";
import { useAppSelector } from "app/hooks";
import { useState } from "react";

const ConnectionWidget: React.FC = () => {

	const connection = useAppSelector(state => state.connection);

	return (
		<List>
			<ListItem>
				<ListItemText primary='Host' secondary={connection.host} />
			</ListItem>
			<ListItem>
				<ListItemText primary='Port' secondary={connection.port} />
			</ListItem>
		</List>
	)
}

const TriggerWordsWidget: React.FC = () => {

	const triggerWords = useAppSelector(state => state.triggerWords.trigger);

	switch (triggerWords.mode) {
		case 'list':
			return (
				<List>
					{triggerWords.words.map((word, index) => (
						<ListItem key={index}>
							<ListItemText primary={word} />
						</ListItem>
					))}
				</List>
			)
		case 'regex':
			return (
				<Typography>{triggerWords.regex}</Typography>
			)
	};
}

const Settings = () => {

	return (
		<Container sx={{ my: 5 }}>
			<Stack>
				<Typography variant='h2'>
					Settings
				</Typography>
				<Stack gap={3} sx={{pt: 3}}>

				<Paper sx={{ p : 3 }}>
					<Typography variant='h6'>
						Connection
					</Typography>
					<ConnectionWidget/>
				</Paper>
				<Paper sx={{ p: 3 }}>
					<Typography variant='h6'>
						Trigger words
						<TriggerWordsWidget/>
					</Typography>
				</Paper>
				</Stack>
			</Stack>
		</Container>
	)
}

export default Settings;
