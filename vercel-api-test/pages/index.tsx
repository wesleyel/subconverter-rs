import React, { useState } from 'react';
import Head from 'next/head';
import { Box, CssBaseline, ThemeProvider, createTheme, Typography, AppBar, Toolbar, Button, Dialog, DialogTitle, DialogContent, DialogContentText, DialogActions, TextField } from '@mui/material';
import Split from 'react-split';
import FileExplorer from '../components/FileExplorer';
import CodeEditor from '../components/CodeEditor';
import { loadGitHubDirectory, LoadDirectoryResult } from '../lib/api-client';

// Create a light theme
const theme = createTheme({
    palette: {
        mode: 'light',
    },
});

// Add this new import for the panic test
async function testPanicHandling() {
    try {
        const response = await fetch('/api/admin/debug-panic', {
            method: 'POST',
        });

        const result = await response.json();
        return result;
    } catch (error) {
        console.error('Error testing panic handling:', error);
        return { error: String(error) };
    }
}

export default function Home() {
    const [selectedFile, setSelectedFile] = useState<string | null>(null);
    const [debugOpen, setDebugOpen] = useState(false);
    const [debugResult, setDebugResult] = useState<string>('');
    const [loadPath, setLoadPath] = useState<string>('');
    const [isLoading, setIsLoading] = useState(false);

    const handleFileSelect = (filePath: string) => {
        setSelectedFile(filePath);
    };

    const handleLoadGitHubDirectory = async () => {
        if (!loadPath.trim()) {
            setDebugResult('Please enter a directory path to load');
            setDebugOpen(true);
            return;
        }

        setIsLoading(true);
        try {
            const result = await loadGitHubDirectory(loadPath);

            // Format the result in a readable way
            const formattedResult = {
                summary: `Successfully loaded ${result.successful_files} of ${result.total_files} files`,
                totalSize: result.loaded_files.reduce((sum, file) => sum + file.size, 0),
                files: result.loaded_files.map(file => ({
                    path: file.path,
                    size: `${(file.size / 1024).toFixed(2)} KB`
                }))
            };

            setDebugResult(JSON.stringify(formattedResult, null, 2));
            setDebugOpen(true);
        } catch (error) {
            setDebugResult(String(error));
            setDebugOpen(true);
        } finally {
            setIsLoading(false);
        }
    };

    const handleTestPanic = async () => {
        setIsLoading(true);
        try {
            const result = await testPanicHandling();
            setDebugResult(JSON.stringify(result, null, 2));
            setDebugOpen(true);
        } catch (error) {
            setDebugResult(String(error));
            setDebugOpen(true);
        } finally {
            setIsLoading(false);
        }
    };

    return (
        <ThemeProvider theme={theme}>
            <CssBaseline />
            <Head>
                <title>VFS Explorer</title>
                <meta name="description" content="Virtual File System Explorer" />
                <link rel="icon" href="/favicon.ico" />
            </Head>

            <Box sx={{ display: 'flex', flexDirection: 'column', height: '100vh' }}>
                <AppBar position="static" color="primary" elevation={0}>
                    <Toolbar>
                        <Typography variant="h6" component="div" sx={{ flexGrow: 1 }}>
                            VFS Explorer
                        </Typography>
                        <TextField
                            size="small"
                            placeholder="Directory path"
                            value={loadPath}
                            onChange={(e) => setLoadPath(e.target.value)}
                            sx={{
                                mr: 2,
                                backgroundColor: 'rgba(255, 255, 255, 0.15)',
                                borderRadius: 1,
                                input: { color: 'white' },
                                width: '200px'
                            }}
                        />
                        <Button
                            color="inherit"
                            onClick={handleLoadGitHubDirectory}
                            disabled={isLoading}
                            sx={{ mr: 2 }}
                        >
                            {isLoading ? 'Loading...' : 'Load GitHub Directory'}
                        </Button>
                        <Button
                            color="error"
                            variant="contained"
                            onClick={handleTestPanic}
                            disabled={isLoading}
                        >
                            Test Stack Trace
                        </Button>
                    </Toolbar>
                </AppBar>

                <Box sx={{ flexGrow: 1, overflow: 'hidden' }}>
                    <Split
                        sizes={[20, 80]}
                        minSize={200}
                        expandToMin={false}
                        gutterSize={10}
                        gutterAlign="center"
                        snapOffset={30}
                        dragInterval={1}
                        direction="horizontal"
                        cursor="col-resize"
                        style={{ display: 'flex', flexDirection: 'row', height: '100%' }}
                    >
                        <Box sx={{ height: '100%', overflow: 'auto', borderRight: '1px solid #eee' }}>
                            <FileExplorer onFileSelect={handleFileSelect} />
                        </Box>
                        <Box sx={{ height: '100%', overflow: 'hidden' }}>
                            <CodeEditor filePath={selectedFile} />
                        </Box>
                    </Split>
                </Box>
            </Box>

            <Dialog open={debugOpen} onClose={() => setDebugOpen(false)} maxWidth="md" fullWidth>
                <DialogTitle>GitHub Directory Load Results</DialogTitle>
                <DialogContent>
                    <DialogContentText component="pre" sx={{ whiteSpace: 'pre-wrap', fontFamily: 'monospace' }}>
                        {debugResult}
                    </DialogContentText>
                </DialogContent>
                <DialogActions>
                    <Button onClick={() => setDebugOpen(false)}>Close</Button>
                </DialogActions>
            </Dialog>
        </ThemeProvider>
    );
} 