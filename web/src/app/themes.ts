import { createTheme } from "@mui/material";


export const theme = createTheme({
	palette: {
		mode: 'dark',
		primary: {
			main: "#111",
		},
		secondary: {
			main: "#311",
			contrastText: '#fff'
		},
		background: {
			default: "#334",
		}
	}
})

export const discord = createTheme({
	palette: {
		primary: {
			main: "#5865F2"
		},
		secondary: {
			main: "#23272A"
		}
	}
})