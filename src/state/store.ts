import { configureStore } from "@reduxjs/toolkit";
import contextMenu from "./slices/contextMenuSlice";

export const store = configureStore({
    reducer: {
        contextMenu,
    }
})

export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;
