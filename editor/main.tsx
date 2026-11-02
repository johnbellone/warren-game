import React from 'react';
import { MantineProvider } from '@mantine/core';
import { AppShell, Header, Navbar, Text } from '@mantine/core';

function App() {
  return (
    <MantineProvider>
      <AppShell
        header={{ height: 60 }}
        navbar={{ width: 300, breakpoint: 'sm' }}
      >
        <Header height={60} p="md">
          <Text size="xl">Warren Game Editor</Text>
        </Header>
        <Navbar p="md">
          <Text size="lg">Editor Interface</Text>
        </Navbar>
      </AppShell>
    </MantineProvider>
  );
}

export default App;