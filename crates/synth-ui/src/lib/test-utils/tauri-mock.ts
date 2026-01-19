/**
 * Tauri API mocking utilities for testing
 *
 * This module provides utilities to mock @tauri-apps/api/core invoke() function
 * for testing components and stores that interact with the Tauri backend.
 */

import { vi, type Mock } from 'vitest';

// Mock Tauri invoke function
export const mockInvoke: Mock = vi.fn();

// Type for invoke command handlers
type InvokeHandler = (args?: unknown) => unknown | Promise<unknown>;
const invokeHandlers = new Map<string, InvokeHandler>();

/**
 * Setup function to mock @tauri-apps/api/core module.
 * Call this in your test file's top-level scope or in beforeAll.
 */
export function setupTauriMock() {
	vi.mock('@tauri-apps/api/core', () => ({
		invoke: mockInvoke
	}));

	// Configure mockInvoke to use handlers or throw for unmocked commands
	mockInvoke.mockImplementation((command: string, args?: unknown) => {
		const handler = invokeHandlers.get(command);
		if (handler) {
			return Promise.resolve(handler(args));
		}
		return Promise.reject(new Error(`Unmocked Tauri command: ${command}`));
	});
}

/**
 * Helper to setup a specific invoke response for a command.
 *
 * @param command - The Tauri command name (e.g., 'get_config', 'set_config')
 * @param response - The response to return when this command is invoked
 */
export function mockInvokeResponse(command: string, response: unknown) {
	invokeHandlers.set(command, () => response);
}

/**
 * Helper to setup a dynamic invoke handler for a command.
 *
 * @param command - The Tauri command name
 * @param handler - Function that receives args and returns the response
 */
export function mockInvokeHandler(command: string, handler: InvokeHandler) {
	invokeHandlers.set(command, handler);
}

/**
 * Helper to make a command throw an error.
 *
 * @param command - The Tauri command name
 * @param error - The error message or Error object
 */
export function mockInvokeError(command: string, error: string | Error) {
	invokeHandlers.set(command, () => {
		throw typeof error === 'string' ? new Error(error) : error;
	});
}

/**
 * Reset all mocks and handlers between tests.
 * Call this in afterEach or beforeEach.
 */
export function resetTauriMock() {
	mockInvoke.mockClear();
	invokeHandlers.clear();
}

/**
 * Clear all handlers but keep mock in place.
 */
export function clearTauriHandlers() {
	invokeHandlers.clear();
}

/**
 * Get the number of times invoke was called with a specific command.
 */
export function getInvokeCallCount(command: string): number {
	return mockInvoke.mock.calls.filter(([cmd]) => cmd === command).length;
}

/**
 * Get all arguments passed to invoke for a specific command.
 */
export function getInvokeArgs(command: string): unknown[] {
	return mockInvoke.mock.calls
		.filter(([cmd]) => cmd === command)
		.map(([, args]) => args);
}

/**
 * Assert that invoke was called with a specific command.
 */
export function expectInvoked(command: string) {
	const calls = mockInvoke.mock.calls.filter(([cmd]) => cmd === command);
	if (calls.length === 0) {
		throw new Error(`Expected Tauri invoke to be called with command "${command}", but it was not called.`);
	}
}

/**
 * Assert that invoke was called with a specific command and args.
 */
export function expectInvokedWith(command: string, args: unknown) {
	const calls = mockInvoke.mock.calls.filter(([cmd]) => cmd === command);
	if (calls.length === 0) {
		throw new Error(`Expected Tauri invoke to be called with command "${command}", but it was not called.`);
	}
	const matchingCall = calls.find(([, callArgs]) => JSON.stringify(callArgs) === JSON.stringify(args));
	if (!matchingCall) {
		throw new Error(
			`Expected Tauri invoke to be called with command "${command}" and args ${JSON.stringify(args)}, ` +
			`but it was called with: ${calls.map(([, a]) => JSON.stringify(a)).join(', ')}`
		);
	}
}
