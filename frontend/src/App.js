import './App.css';
import 'leaflet/dist/leaflet.css';
import 'leaflet-contextmenu';
import 'leaflet-contextmenu/dist/leaflet.contextmenu.css';

import React, { useRef, useState } from 'react';
import { MapContainer, LayerGroup, LayersControl, TileLayer, Marker, Tooltip, Popup, Polyline, useMapEvents } from 'react-leaflet';
import { MgrsGraticule } from 'react-leaflet-mgrs-graticule';
import { Point } from '@ngageoint/grid-js'
import { MGRS } from '@ngageoint/mgrs-js'
import L from 'leaflet';
import axios from 'axios';
import { Popup as PopupJS } from 'reactjs-popup';

import iconMap from './map.png'
import iconMarker from './marker.png'
import iconCenter from './center.png'
import iconRuler from './ruler.png'
import iconZoomIn from './zoom-in.png'
import iconZoomOut from './zoom-out.png'

delete L.Icon.Default.prototype._getIconUrl;

L.Icon.Default.mergeOptions({
    iconRetinaUrl: require('leaflet/dist/images/marker-icon-2x.png'),
    iconUrl: require('leaflet/dist/images/marker-icon.png'),
    shadowUrl: require('leaflet/dist/images/marker-shadow.png')
});

const mgrsGraticuleName = 'MGRS';
const overlayEnabled = true;
const graticuleOptions = {
  font: '18px Courier New',
  gridFont: '18px Courier New',
};
const defaultCoordinates = [49.22744583038021, 16.574301072757255]
const defaultPathOptions = {
  fillCollor: 'blue',
  dashArray: '20, 20',
}

// Post Coordinates To The Backend
async function postCoordinates(mgrsA, mgrsB) {
  // Example values for coordinates
  const x1 = mgrsA.easting;
  const y1 = mgrsA.northing;
  const x2 = mgrsB.easting;
  const y2 = mgrsB.northing;

  // Convert values to byte arrays (Vec<u8> equivalent)
  const x1_encoded = new Uint8Array(new Int32Array([x1]).buffer);
  const y1_encoded = new Uint8Array(new Int32Array([y1]).buffer);
  const x2_encoded = new Uint8Array(new Int32Array([x2]).buffer);
  const y2_encoded = new Uint8Array(new Int32Array([y2]).buffer);

  // Construct the payload
  const payload = {
    coordinate_a: {
      x: Array.from(x1_encoded),
      y: Array.from(y1_encoded),
    },
    coordinate_b: {
      x: Array.from(x2_encoded),
      y: Array.from(y2_encoded),
    }
  };

  try {
    // Make the POST request using Axios
    const response = await axios.post('http://127.0.0.1:12345/api/admin/calc/dist', payload);
  
    // Return the object
    return response.data.data
  } catch (error) {
    throw error;
  }
}

// Helper function to convert byte array to float
function convertByteArrayToFloat(byteArray) {
  // Create a Uint8Array from the byte array
  const uint8Array = new Uint8Array(byteArray);

  // Create a DataView to interpret the Uint8Array as a float
  const dataView = new DataView(uint8Array.buffer);

  // Get the float value from the DataView (assuming little-endian format)
  return dataView.getFloat32(0, true);  // true for little-endian, false for big-endian
}

// Convert array of bytes to hex string
function toHexString(byteArray) {
  return Array.from(byteArray, function(byte) {
    return ('0' + (byte & 0xFF).toString(16)).slice(-2);
  }).join('')
}


// Main App
function App() {
  // Map referrence
  const mapRef = useRef(null);

  // Modal
  const [isOpen, setIsOpen] = useState(false);
  
  // Context menu
  const [isDisabled, setIsDisabled] = useState(false);

  // First marker
  const [enableFirstMarker, setEnableFirstMarker] = useState(false);
  const [locationFirstMarker, setLocationFirstMarker] = useState(defaultCoordinates)
  const [stringFirstMarkerWGS84, setStringFirstMarkerWGS84] = useState(null)
  const [stringFirstMarkerMGRS, setStringFirstMarkerMGRS] = useState(null)

  // Second marker
  const [enableSecondMarker, setEnableSecondMarker] = useState(false);
  const [locationSecondMarker, setLocationSecondMarker] = useState(defaultCoordinates)
  const [stringSecondMarkerWGS84, setStringSecondMarkerWGS84] = useState(null)
  const [stringSecondMarkerMGRS, setStringSecondMarkerMGRS] = useState(null)

  // Results
  const [distance, setDistance] = useState(null);
  const [computationMethod, setComputationMethod] = useState(null);
  const [digest, setDigest] = useState(null);
  const [digestStatus, setDigestStatus] = useState(null);

  // Default Context Menu Items
  const DefaultContextMenuItems = [
    {
      separator: true,
    },
    {
      text: !enableFirstMarker ? "Place Marker A" : "Remove Marker A",
      callback: firstMarker,
      icon: iconMarker,
      disabled: isDisabled,
    },
    {
      text: !enableSecondMarker ? "Place Marker B" : "Remove Marker B",
      callback: secondMarker,
      icon: iconMarker,
      disabled: isDisabled,
    },
    {
      text: "Calculate Distance",
      callback: calculateDistance,
      icon: iconRuler,
      disabled: isDisabled || !(enableFirstMarker && enableSecondMarker),
    },
    {
      separator: true,
    },
    {
      text: 'Center Map Here',
      icon: iconCenter,
      callback: centerMapHere,
      disabled: isDisabled,
    },
    {
      text: 'Zoom In',
      icon: iconZoomIn,
      callback: zoomIn,
      disabled: isDisabled,
    },
    {
      text: 'Zoom Out',
      icon: iconZoomOut,
      callback: zoomOut,
      disabled: isDisabled,
    },
  ]

  const MapEvents = () => {
    useMapEvents({
      contextmenu(e) {
        // Get current coordinates
        let point = Point.point(e.latlng.lng, e.latlng.lat);
        let mgrs = MGRS.from(point)

        // Format
        let formattedGPS = `WGS84: ${e.latlng.lat.toFixed(5)}°N ${e.latlng.lng.toFixed(5)}°E`;
        let mgrsString = mgrs.toString();
        let formattedMGRS = `MGRS: ${mgrsString.slice(0, 5)} ${mgrsString.slice(5, 10)} ${mgrsString.slice(10)}`

        // Remove all context menu items and add the new one with default ones
        mapRef.current.contextmenu.removeAllItems();
        mapRef.current.contextmenu.addItem({
          text: "Coordinates",
          icon: iconMap,
          disabled: isDisabled,
        });
        mapRef.current.contextmenu.addItem({
          text: formattedGPS,
          disabled: isDisabled,
        });
        mapRef.current.contextmenu.addItem({
          text: formattedMGRS,
          disabled: isDisabled,
        });
        DefaultContextMenuItems.forEach(item => {
          mapRef.current.contextmenu.addItem(item);
        });
      },
    });
    return false;
  }

  // First Marker Function
  function firstMarker(e) {
    if (mapRef.current) {
      // Get current coordinates
      let point = Point.point(e.latlng.lng, e.latlng.lat);
      let mgrs = MGRS.from(point)

      // Format
      let formattedGPS = `WGS84: ${e.latlng.lat.toFixed(5)}°N ${e.latlng.lng.toFixed(5)}°E`;
      let mgrsString = mgrs.toString();
      let formattedMGRS = `MGRS: ${mgrsString.slice(0, 5)} ${mgrsString.slice(5, 10)} ${mgrsString.slice(10)}`

      setEnableFirstMarker(!enableFirstMarker);
      setLocationFirstMarker([e.latlng.lat, e.latlng.lng]);
      setStringFirstMarkerWGS84(formattedGPS);
      setStringFirstMarkerMGRS(formattedMGRS);
    }
  }

  // Second Marker Function
  function secondMarker(e) {
    if (mapRef.current) {
      // Get current coordinates
      let point = Point.point(e.latlng.lng, e.latlng.lat);
      let mgrs = MGRS.from(point)

      // Format
      let formattedGPS = `WGS84: ${e.latlng.lat.toFixed(5)}°N ${e.latlng.lng.toFixed(5)}°E`;
      let mgrsString = mgrs.toString();
      let formattedMGRS = `MGRS: ${mgrsString.slice(0, 5)} ${mgrsString.slice(5, 10)} ${mgrsString.slice(10)}`

      setEnableSecondMarker(!enableSecondMarker);  
      setLocationSecondMarker([e.latlng.lat, e.latlng.lng]);
      setStringSecondMarkerWGS84(formattedGPS);
      setStringSecondMarkerMGRS(formattedMGRS);
    }
  }

  // TODO:
  // Calculate Distance Function
  async function calculateDistance(e) {
    setIsDisabled(true);
    
    // Get current coordinates
    let pointA = Point.point(locationFirstMarker[1], locationFirstMarker[0]);
    let mgrsA = MGRS.from(pointA);
    let pointB = Point.point(locationSecondMarker[1], locationSecondMarker[0]);
    let mgrsB = MGRS.from(pointB);
    
    try {
      // Post The Coordinates
      let response = await postCoordinates(mgrsA, mgrsB);
      console.log(response);

      // Extract the distance array from the response
      const distanceArray = response.distance;

      // Convert the distance array back to a float
      const distanceFloat = convertByteArrayToFloat(distanceArray);

      // Convert the distance array to hex string
      const distanceString = toHexString(distanceArray);
      // const distanceString = "test";

      // Convert the array of bytes into hex string
      const digestString = toHexString(response.digest);
      // const digestString = "test";

      // Get the digest of the distance hex string
      const CryptoJS = require('crypto-js');
      const digestDistanaceString = CryptoJS.SHA256(CryptoJS.enc.Hex.parse(distanceString)).toString();

      let digestStatus = "";
      if (digestString.localeCompare(digestDistanaceString) === 0) {
        digestStatus = "OK";
      } else {
        digestStatus = "INVALID"
      }

      // Set Results
      setDistance(distanceFloat);
      setComputationMethod(response.comment);
      setDigest(digestString);
      setDigestStatus(digestStatus);

      // Open Modal
      setIsOpen(true);
    } catch (error) {
      console.log(error);
      alert("Error occurred. Please try again.")
    }
   
    setIsDisabled(false);
  }
  
  // Center Map Function
  function centerMapHere(e) {
    if (mapRef.current) {
      mapRef.current.panTo(e.latlng);
    }
  }
  
  // Zoom In Function
  function zoomIn(e) {
    if (mapRef.current) {
      mapRef.current.zoomIn();
    }
  }

  // Zoom Out Function
  function zoomOut(e) {
    if (mapRef.current) {
      mapRef.current.zoomOut();
    }
  }

  return (
    <MapContainer
      center={defaultCoordinates}
      zoom={18}
      minZoom={3}
      maxZoom={18}
      maxNativeZoom={15}
      maxBounds={[
        [-90, -180],
        [90, 180],
      ]}
      style={{height: "100vh"}}
      contextmenu={true}
      contextmenuItems={DefaultContextMenuItems}
      ref={mapRef}
    >
      <PopupJS open={isOpen} modal closeOnDocumentClick onClose={() => { setIsOpen(false) }}>
        <div className="modal">
          <div className="header">Distance Calculation Results</div>
          <div className="content">
            <div style={{ display: 'grid', gridTemplateColumns: 'auto 1fr', gap: '10px' }}>
              <div>Distance [m]:</div>
              <div>{distance}</div>
              <div>Computation Method:</div>
              <div>{computationMethod}</div>
              <div>Digest (SHA-256):</div>
              <div style={{ wordBreak: 'break-all' }}>{digest}</div>
              <div>Digest Status:</div>
              <div>{digestStatus}</div>
            </div>
          </div>
        </div>
      </PopupJS>
      { enableFirstMarker &&
        <Marker position={locationFirstMarker} >
          <Tooltip direction='bottom' offset={[-16, 30]} permanent>{"A"}</Tooltip>
          <Popup>
            {stringFirstMarkerWGS84} <br /> {stringFirstMarkerMGRS}
          </Popup>
        </Marker>
      }
      { enableSecondMarker &&
        <Marker position={locationSecondMarker} >
          <Tooltip direction='bottom' offset={[-16, 30]} permanent>{"B"}</Tooltip>
          <Popup>
            {stringSecondMarkerWGS84} <br /> {stringSecondMarkerMGRS}
          </Popup>
        </Marker>
      }
      {
        enableFirstMarker && enableSecondMarker &&
        <Polyline pathOptions={defaultPathOptions} positions={[locationFirstMarker, locationSecondMarker]} />
      }
      <MapEvents/>
      <LayersControl position="topright">
        <LayersControl.BaseLayer name="ESRI Satellite">
          <TileLayer
            url="https://services.arcgisonline.com/arcgis/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}"
            attribution='&copy; <a href="https://wiki.openstreetmap.org/wiki/Esri"></a> contributors'
          />
        </LayersControl.BaseLayer>
        {/* <LayersControl.BaseLayer name="ESRI Clarity">
          <TileLayer
            url="https://clarity.maptiles.arcgis.com/arcgis/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}"
            attribution='&copy; <a href="https://wiki.openstreetmap.org/wiki/Esri"></a> contributors'
          />
        </LayersControl.BaseLayer> */}
        <LayersControl.BaseLayer checked name="OSM">
          <TileLayer
            url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
            attribution='&copy; <a href="http://osm.org/copyright">OpenStreetMap</a> contributors'
          />
        </LayersControl.BaseLayer>
        {/* <LayersControl.BaseLayer name="OSM Topo">
          <TileLayer url="https://{s}.tile.opentopomap.org/{z}/{x}/{y}.png" attribution="OSM" />
        </LayersControl.BaseLayer> */}
        <LayersControl.Overlay checked={overlayEnabled} name={mgrsGraticuleName}>
          <LayerGroup>
            <MgrsGraticule name={mgrsGraticuleName} checked={overlayEnabled} options={graticuleOptions} />
          </LayerGroup>
        </LayersControl.Overlay>
      </LayersControl>
    </MapContainer>
  );
}

export default App;
