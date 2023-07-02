import Input, {InputSize} from "../ui/Input";
import {useState} from "react";
import Button, {ButtonSize} from "../ui/Button";

interface Props {
    title: string;
    submitName: string;
    onSubmit: (value: string) => unknown;
    shown: boolean;
    setShown: (shown: boolean) => unknown;
}

export default function InputModal({ shown, setShown, title, onSubmit, submitName }: Props) {
    const [inputValue, setInputValue] = useState("");

    if (shown) {
        return (
            <div className="absolute w-full h-full z-10" style={{
                backgroundColor: "rgba(0,0,0, 0.4)"
            }}>
                <div className="flex justify-around flex-col absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 bg-darker rounded-lg w-60 h-32 z-20 border-gray-700 border-1">
                    <h3 className="text-center">{title}</h3>
                    <Input value={inputValue} setValue={setInputValue} size={InputSize.Tiny} className="block mr-auto ml-auto text-center w-48" />

                    <div className="flex justify-center space-x-2">
                        <Button onClick={() => {
                            setInputValue("");
                            setShown(false);
                        }} size={ButtonSize.Small}>Cancel</Button>
                        <Button onClick={() => {
                            setInputValue("");
                            if (inputValue.length > 1) {
                                onSubmit(inputValue);
                                setShown(false);
                                return;
                            }

                            alert("Input must be at least 2 characters long.");
                        }} size={ButtonSize.Small}>{submitName}</Button>
                    </div>
                </div>
            </div>
        )
    } else {
        return <></>
    }
}
