export default function myInitializer() {
  return {
    onStart: () => {
      console.log('Loading...');
      console.time('initializer');
    },
    onProgress: ({ current, total }) => {
      if (!total) {
        console.log('Loading...', current, 'bytes');
      } else {
        console.log('Loading...', Math.round((current / total) * 100), '%');
      }
    },
    onComplete: () => {
      console.log('Loading... done!');
      console.timeEnd('initializer');
    },
    onSuccess: () => {
      console.log('Loading... successful!');
    },
    onFailure: (error) => {
      console.warn('Loading... failed!', error);
    },
  };
}
