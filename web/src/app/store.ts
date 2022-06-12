import { configureStore } from "@reduxjs/toolkit";
import { apiSlice } from "../features/api-slice";

export const store = configureStore({
	reducer: {
		[apiSlice.reducerPath]: apiSlice.reducer
	}
})

export type AppDispatch = typeof store.dispatch;

export type RootState = ReturnType<typeof store.getState>;