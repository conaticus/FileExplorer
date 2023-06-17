import {Dispatch, SetStateAction} from "react";

export enum InputSize {
    Tiny,
    Large,
}


interface Props {
    value: string;
    setValue: Dispatch<SetStateAction<string>>;
    placeholder?: string;
    onSubmit?: () => any;
    size: InputSize;
    className?: string;
    disabled?: boolean;
}

export default function Input({ value, setValue, placeholder, onSubmit, size, className, disabled }: Props) {
    let styles = `outline-none bg-darker border-gray-500 border-1 rounded-md focus:border-gray-300 p-2 disabled:opacity-25 ${className + " " || ""}`;

    switch (size) {
        case InputSize.Large:
            styles += "h-10 w-72";
            break;
        case InputSize.Tiny:
            styles += "h-6 w-20 text-center";
            break;
        default: break;
    }

    return (
       <input
           disabled={disabled}
           value={value}
           onChange={(e) => setValue(e.target.value)}
           className={styles}
           placeholder={placeholder}
           onSubmit={onSubmit}
       />
   )
}