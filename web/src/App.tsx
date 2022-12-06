import { CssBaseline } from '@mui/material';
import { ThemeProvider } from '@mui/system';
import { useState } from 'react'
import { Provider } from 'react-redux';
import { BrowserRouter, Route, Routes } from 'react-router-dom'
import { store } from 'app/store';
import { theme } from 'app/themes';
import Header from 'app/Header';
import Main from 'app/Main';
import Settings from 'features/Settings';


function App() {
	return (
		<Provider store={store}>
			<ThemeProvider theme={theme}>
				<CssBaseline/>
				<BrowserRouter>
					<Header/>
					<Routes>
						<Route path="/" element={<Main/>} />
						<Route path="/settings" element={<Settings/>} />
					</Routes>
				</BrowserRouter>
			</ThemeProvider>
		</Provider>
	)
}

export default App;
