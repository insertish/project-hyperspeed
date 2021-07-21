document.addEventListener('DOMContentLoaded', (event) => {
    const script = document.createElement('script');
    script.defer = true;
    script.async = true;
    
    script.onload = () => {
        document.querySelectorAll('pre code')
            .forEach((el) => hljs.highlightElement(el));
        
        const observer = new MutationObserver(list => {
            for (const entry of list) {
                entry.target
                    .querySelectorAll('pre code')
                    .forEach((el) => hljs.highlightElement(el));
            }
        });

        observer.observe(document.body, { childList: true, subtree: true });
    }
    
    script.src = '/highlight.min.js';
    document.body.appendChild(script);

    const style = document.createElement('link');
    style.rel = 'stylesheet';
    style.type = 'text/css';
    style.href = '/github-dark.min.css';

    document.body.appendChild(style);
});
