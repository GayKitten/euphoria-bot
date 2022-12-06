import {createSlice, PayloadAction} from '@reduxjs/toolkit';

type TriggerWords = {
	mode: 'list';
	words: string[];
} | {
	mode: 'regex';
	regex: string;
}

interface Wrapper {
	trigger: TriggerWords;
}

const initialState: Wrapper = {
	trigger: {
		mode: 'list',
		words: ['good girl'],
	}
};

const triggerWordsSlice = createSlice({
	name: 'triggerWords',
	initialState,
	reducers: {
		setState(state, action: PayloadAction<TriggerWords>) {
			state.trigger = action.payload;
		}
	}
});

export const {setState} = triggerWordsSlice.actions;
export default triggerWordsSlice.reducer;