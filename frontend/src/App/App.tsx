import { type FunctionComponent } from 'react';
import { RetryErrorBoundary } from '../components/RetryBoundary';
import { AllPackagesLoader } from '../components/AllPackagesLoader';
import { LoginBoundary } from '../components/LoginBoundary';

const App: FunctionComponent = () => (
    <LoginBoundary>
        <RetryErrorBoundary>
            <AllPackagesLoader/>
        </RetryErrorBoundary>
    </LoginBoundary>
)

export default App
