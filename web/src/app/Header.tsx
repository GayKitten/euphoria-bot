import { AppBar, Avatar, Button, Container, IconButton, Menu, MenuItem, Paper, ThemeProvider, Toolbar, Typography } from "@mui/material";
import { useEffect, useRef, useState } from "react";
import { Link, useSearchParams } from "react-router-dom";
import { discord } from "app/themes";

import { useLoginMutation, useMeQuery, User } from "features/api-slice";

const GuestHeader = () => {

	return (
		<AppBar position="static">
			<Toolbar>
				<Typography
					sx={{ flexGrow: 1 }}
					variant="h4"
					component={Link}
					to="/"
				>
					Euphoria
				</Typography>
				<ThemeProvider theme={discord}>
					<Button
						variant="contained"
						href="https://discord.com/api/oauth2/authorize?client_id=947195490322235482&redirect_uri=https%3A%2F%2Flocalhost%3A2069%2F&response_type=code&scope=identify"
					>
						Log in with Discord
					</Button>
				</ThemeProvider>
			</Toolbar>
		</AppBar>
	)
}

const UserHeader: React.FC<{ user: User }> = ({ user }) => {

	const [menuOpen, setMenuOpen] = useState(false);
	const menuRef = useRef<HTMLDivElement>(null);

	return (
		<AppBar position="static">
			<Toolbar>
				<Typography
					sx={{ flexGrow: 1 }}
					variant="h4"
					component={Link}
					to="/"
				>
					Euphoria
				</Typography>
				<Typography>
					{user.username}
				</Typography>
				<IconButton
					onClick={() => setMenuOpen(true)}
				>
					<Avatar
						ref={menuRef}
						alt={user.username}
						src={'https://cdn.discordapp.com/avatars/' + user.id + '/' + user.avatar + '.png'}
					/>
				</IconButton>
				<Menu
					anchorEl={menuRef.current}
					open={menuOpen}
					onClose={() => setMenuOpen(false)}
				>
					<MenuItem component={Link} to="/settings">
						Settings
					</MenuItem>
					<MenuItem>
						Log out
					</MenuItem>
				</Menu>
			</Toolbar>
		</AppBar>
	)
}

const Header = () => {
	const [searchParam] = useSearchParams()
	const [login] = useLoginMutation();

	const { data } = useMeQuery();

	useEffect(() => {
		const code = searchParam.get("code")
		if (code === null) {
			return;
		}
		login(code).then(() => {
			searchParam.delete("code");
		})
	}, [searchParam])

	if (data !== undefined && data.status === 'LoggedIn') return <UserHeader user={data.user} />
	return <GuestHeader/>
}

export default Header;