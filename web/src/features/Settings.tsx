import { Container, List, ListItem, ListItemText, Paper, Stack, Typography } from "@mui/material";
import { useState } from "react";

type TriggerWords = {
	mode: 'list',
	words: string[],
} | {
	mode: 'regex',
	regex: string,
}

const TriggerWordsWidget: React.FC<{ triggerWords: TriggerWords }> = ({ triggerWords }) => {
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
	}
}

const Settings = () => {
	const [triggerWords, setTriggerWords] = useState<TriggerWords>({
		mode: 'list',
		words: ['good girl'],
	})
	return (
		<Container sx={{ my: 5 }}>
			<Stack>
				<Typography variant='h2'>
					Settings
				</Typography>
				<Paper sx={{ p: 3 }}>
					<Typography variant='h6'>
						Trigger words
					</Typography>
				</Paper>
			</Stack>
		</Container>
	)
}

export default Settings;
