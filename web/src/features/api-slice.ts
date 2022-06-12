import { createApi, fetchBaseQuery } from '@reduxjs/toolkit/query/react';

export interface UserData {
	username: string,
	pictureUrl: string,
}

export const apiSlice = createApi({
	reducerPath: 'api',
	baseQuery: fetchBaseQuery({
		baseUrl: 'https://localhost:4000'
	}),
	endpoints(builder) {
		return {
			login: builder.mutation<UserData, string>({
				query: (code) => ({
					url: '/login'
				}),
			})
		}
	}
})

export const {
	useLoginMutation
} = apiSlice;