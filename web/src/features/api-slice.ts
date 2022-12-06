import { createApi, fetchBaseQuery } from '@reduxjs/toolkit/query/react';

export interface User {
	id: string,
	username: string,
	avatar: string,
}

export type UserStatus = {
	status: 'LoggedIn';
	user: User;
} | {
	status: 'LoggedOut';
}

export const apiSlice = createApi({
	reducerPath: 'api',
	baseQuery: fetchBaseQuery({
		baseUrl: 'api',
		credentials: 'include',
	}),
	tagTypes: ['user'],
	endpoints(builder) {
		return {
			login: builder.mutation<void, string>({
				query: (code) => ({
					url: '/login',
					params: { code },
					method: 'POST',
				}),
				invalidatesTags: ['user'],
			}),
			me: builder.query<UserStatus, void>({
				query: () => ({
					url: '/me',
					method: 'GET',
				}),
				providesTags: ['user'],
			}),
		}
	}
})

export const {
	useLoginMutation,
	useMeQuery,
} = apiSlice;