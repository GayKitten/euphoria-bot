import { configureStore, getDefaultMiddleware } from "@reduxjs/toolkit";
import connectionSlice from "features/connections/connection-slice";
import triggerWordsSlice from "features/settings/trigger-words";
import { apiSlice } from "features/api-slice";

export const store = configureStore({
	reducer: {
		[apiSlice.reducerPath]: apiSlice.reducer,
		connection: connectionSlice,
		triggerWords: triggerWordsSlice,
	},
	middleware: (getDefaultMiddleware) => getDefaultMiddleware().concat(apiSlice.middleware)
})

export type AppDispatch = typeof store.dispatch;

export type RootState = ReturnType<typeof store.getState>;