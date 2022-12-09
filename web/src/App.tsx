import { CssBaseline } from '@mui/material';
import { ThemeProvider } from '@mui/system';
import { useState, createContext } from 'react'
import { Provider } from 'react-redux';
import { BrowserRouter, Route, Routes } from 'react-router-dom'
import { store } from 'app/store';
import { theme } from 'app/themes';
import Header from 'app/Header';
import Main from 'app/Main';
import Settings from 'features/settings/Settings';
import { WSBridge } from 'features/connections/connection-bridge';

interface BridgeContextType {
	bridge: WSBridge | null;
	connect: (url: string) => void;
}

export const BridgeContext = createContext<BridgeContextType>({bridge: null, connect: () => {}});

function App() {

	// make a state for the websocket connection
	const [bridge, setBridge] = useState<WSBridge | null>(null);

	const connect = (url: string) => {
		const bridge = new WSBridge(url, "wss://localhost:2069/api/connect");
		setBridge(bridge);
	}


	return (
		<Provider store={store}>
			<ThemeProvider theme={theme}>
				<BridgeContext.Provider value={{bridge, connect}}>
				<CssBaseline/>
				<BrowserRouter>
					<Header/>
					<Routes>
						<Route path="/" element={<Main/>} />
						<Route path="/settings" element={<Settings/>} />
					</Routes>
				</BrowserRouter>
				</BridgeContext.Provider>
			</ThemeProvider>
		</Provider>
	)
}

export default App;
