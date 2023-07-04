import {ChangeEvent, Dispatch, SetStateAction, KeyboardEvent} from "react";

export enum InputSize {
    Tiny,
    Large,
}


interface Props {
    value: string;
    setValue?: Dispatch<SetStateAction<string>>;
    onChange?: (e: ChangeEvent<HTMLInputElement>) => any;
    placeholder?: string;
    onSubmit?: () => any;
    size: InputSize;
    className?: string;
    disabled?: boolean;
    min?: string;
    max?: string;
}

export default function Input({ value, onChange, setValue, placeholder, onSubmit, size, className, disabled, min, max }: Props) {
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

    function onKeydown({ key, target }: KeyboardEvent<HTMLInputElement>) {
        if (key === "Enter" && onSubmit) { onSubmit(); }
    }

    return (
       <input
           disabled={disabled}
           value={value}
           onChange={(e) => setValue ? setValue(e.target.value) : onChange ? onChange(e) : undefined}
           className={styles}
           placeholder={placeholder}
           onKeyDown={onKeydown}
           min={min}
           max={max}
       />
   )
}