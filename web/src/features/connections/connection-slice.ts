import { createSlice, PayloadAction } from "@reduxjs/toolkit";

interface IntifaceConnectionSettings {
	port: number;
	host: string;

}

const initialState: IntifaceConnectionSettings = {
	port: 12345,
	host: 'localhost',
};

const connectionSlice = createSlice({
	name: "connection",
	initialState,
	reducers: {
		setPort(state, action: PayloadAction<number>) {
			state.port = action.payload;
		},
		setUrl(state, action: PayloadAction<string>) {
			state.host = action.payload;
		},
	},
});

export const { setPort, setUrl } = connectionSlice.actions;
export default connectionSlice.reducer;