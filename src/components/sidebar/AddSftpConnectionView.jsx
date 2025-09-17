
import React, { useState } from 'react';
import Modal from '../common/Modal';
import Button from '../common/Button';

/**
 * AddSftpConnectionView - Modal for adding a new SFTP connection
 * @param {Object} props
 * @param {boolean} props.isOpen - Whether the modal is open
 * @param {Function} props.onClose - Function to close the modal
 * @param {Function} props.onAdd - Function to add the SFTP connection
 */
const AddSftpConnectionView = ({ isOpen, onClose, onAdd }) => {
	const [name, setName] = useState('');
	const [host, setHost] = useState('localhost');
	const [port, setPort] = useState('22');
	const [username, setUsername] = useState('');
	const [password, setPassword] = useState('');
	const [testing, setTesting] = useState(false);
	const [testResult, setTestResult] = useState(null);
	const [error, setError] = useState(null);

	const handleTestConnection = async () => {
		setTesting(true);
		setTestResult(null);
		setError(null);
		try {
			// Use Tauri invoke to test SFTP connection by calling load_dir on "."
			const { invoke } = await import('@tauri-apps/api/core');
			await invoke('load_dir', {
				host,
				port: parseInt(port, 10),
				username,
				password,
				directory: "."
			});
			setTestResult('Connection successful!');
		} catch (e) {
			setTestResult(null);
			setError(e?.toString() || 'Connection failed');
		} finally {
			setTesting(false);
		}
	};

	const handleAdd = () => {
		if (!name.trim() || !host.trim() || !port.trim() || !username.trim()) return;
		onAdd({ name, host, port, username, password });
		setName('');
		setHost('localhost');
		setPort('22');
		setUsername('');
		setPassword('');
		setTestResult(null);
		setError(null);
	};

	const handleClose = () => {
		setName('');
		setHost('localhost');
		setPort('22');
		setUsername('');
		setPassword('');
		setTestResult(null);
		setError(null);
		onClose();
	};

	return (
		<Modal
			isOpen={isOpen}
			onClose={handleClose}
			title="Add SFTP Connection"
			size="sm"
			footer={
				<>
					<Button variant="ghost" onClick={handleClose}>Cancel</Button>
					<Button
						variant="secondary"
						onClick={handleTestConnection}
						disabled={testing || !host.trim() || !port.trim() || !username.trim()}
					>
						{testing ? 'Testing...' : 'Test Connection'}
					</Button>
					<Button
						variant="primary"
						onClick={handleAdd}
						disabled={!name.trim() || !host.trim() || !port.trim() || !username.trim()}
					>
						Add
					</Button>
				</>
			}
		>
			<form onSubmit={e => { e.preventDefault(); handleAdd(); }}>
				<div className="form-group">
					<label htmlFor="sftp-name">Name</label>
					<input
						type="text"
						id="sftp-name"
						className="input"
						value={name}
						onChange={e => setName(e.target.value)}
						placeholder="Connection name"
						autoFocus
					/>
				</div>
				<div className="form-group">
					<label htmlFor="sftp-host">IP / Host</label>
					<input
						type="text"
						id="sftp-host"
						className="input"
						value={host}
						onChange={e => setHost(e.target.value)}
						placeholder="localhost or IP address"
					/>
				</div>
				<div className="form-group">
					<label htmlFor="sftp-port">Port</label>
					<input
						type="number"
						id="sftp-port"
						className="input"
						value={port}
						onChange={e => setPort(e.target.value)}
						min="1"
						max="65535"
					/>
				</div>
				<div className="form-group">
					<label htmlFor="sftp-username">Username</label>
					<input
						type="text"
						id="sftp-username"
						className="input"
						value={username}
						onChange={e => setUsername(e.target.value)}
						placeholder="Username"
					/>
				</div>
				<div className="form-group">
					<label htmlFor="sftp-password">Password</label>
					<input
						type="password"
						id="sftp-password"
						className="input"
						value={password}
						onChange={e => setPassword(e.target.value)}
						placeholder="Password"
					/>
				</div>
				{testResult && <div className="input-hint" style={{ color: 'var(--success)' }}>{testResult}</div>}
				{error && <div className="input-hint" style={{ color: 'var(--danger)' }}>{error}</div>}
			</form>
		</Modal>
	);
};

export default AddSftpConnectionView;
