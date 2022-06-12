import { CssBaseline } from '@mui/material';
import { ThemeProvider } from '@mui/system';
import { useState } from 'react'
import { Provider } from 'react-redux';
import { BrowserRouter, Route, Routes } from 'react-router-dom'
import './App.css'
import Header from './app/Header';
import { store } from './app/store';
import { theme } from './app/themes';


function App() {
	const [count, setCount] = useState(0)

	return (
		<Provider store={store}>
			<ThemeProvider theme={theme}>
				<CssBaseline/>
				<BrowserRouter>
					<Header/>
					<Routes>
						{/*<Route path="/" element={<Main/>}/>*/}
					</Routes>
				</BrowserRouter>
			</ThemeProvider>
		</Provider>
	)
}

export default App
