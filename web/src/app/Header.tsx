import { AppBar, Button, Container, Paper, ThemeProvider, Toolbar, Typography } from "@mui/material";
import { useEffect } from "react";
import { useSearchParams } from "react-router-dom";
import "./../App.css"
import { discord } from "./themes";

import { useLoginMutation } from "../features/api-slice";

const Header = () => {
	const [searchParam] = useSearchParams()
	const [login, {data = {}, }] = useLoginMutation();

	useEffect(() => {
		const code = searchParam.get("code")
		if(code === null) {
			return;
		}
		login(code)
	}, [])

	return (
		<div>
			<AppBar position="static">
				<Toolbar>
					<Typography
						sx={{flexGrow: 1}}
					>
						Euphoria
					</Typography>
					<ThemeProvider theme={discord}>
						<Button
							variant="contained"
							href="https://discord.com/api/oauth2/authorize?client_id=947195490322235482&redirect_uri=https%3A%2F%2Flocalhost%3A3000%2F&response_type=code&scope=identify"
						>
							Log in with Discord
						</Button>
					</ThemeProvider>
				</Toolbar>
			</AppBar>
		</div>
	)
}

export default Header;