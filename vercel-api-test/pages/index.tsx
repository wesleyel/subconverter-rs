import React, { useState } from 'react';
import Head from 'next/head';
import { Box, CssBaseline, ThemeProvider, createTheme, Typography, AppBar, Toolbar, Button, Dialog, DialogTitle, DialogContent, DialogContentText, DialogActions } from '@mui/material';
import Split from 'react-split';
import FileExplorer from '../components/FileExplorer';
import CodeEditor from '../components/CodeEditor';

// Create a light theme
const theme = createTheme({
    palette: {
        mode: 'light',
    },
});

export default function Home() {
    const [selectedFile, setSelectedFile] = useState<string | null>(null);
    const [debugOpen, setDebugOpen] = useState(false);
    const [debugResult, setDebugResult] = useState<string>('');

    const handleFileSelect = (filePath: string) => {
        setSelectedFile(filePath);
    };

    const handleTestEdgeFunctions = async () => {
        try {
            const response = await fetch('/api/test?wasm=true');
            const data = await response.json();
            setDebugResult(JSON.stringify(data, null, 2));
            setDebugOpen(true);
        } catch (error) {
            setDebugResult(String(error));
            setDebugOpen(true);
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
                        <Button color="inherit" onClick={handleTestEdgeFunctions}>
                            Test Edge Functions
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

            <Dialog open={debugOpen} onClose={() => setDebugOpen(false)}>
                <DialogTitle>Edge Function Test Results</DialogTitle>
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